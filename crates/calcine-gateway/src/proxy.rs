//! Forwarding inference requests to `geniex serve`, streaming included.

use std::sync::Arc;
use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::{OwnedSemaphorePermit, mpsc};
use tokio_stream::wrappers::ReceiverStream;

use crate::caller::Caller;
use crate::error::{api_error, upstream_error, upstream_message};
use crate::log::{RequestEntry, RequestLog, SseUsage, Usage};
use crate::security::sanitize;
use crate::state::{ActiveRequest, AppState};

/// One request in flight: where it's logged, when it started, and its turn
/// in the queue (released when this is dropped).
struct Exchange {
    log: Arc<RequestLog>,
    id: u32,
    started: Instant,
    _turn: (OwnedSemaphorePermit, ActiveRequest),
}

impl Exchange {
    fn fail(&self, status: StatusCode, message: &str) -> Response {
        self.log.update(self.id, |entry| {
            entry.status = Some(status.as_u16());
            entry.duration_ms = Some(elapsed_ms(self.started));
            entry.error = Some(message.to_owned());
        });
        api_error(status, "server_error", message)
    }
}

pub async fn inference(
    app: Arc<AppState>,
    caller: Caller,
    path: &'static str,
    body: Bytes,
) -> Response {
    if !caller.can_infer() {
        return api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "this API key can't run models",
        );
    }
    let json: Value = match serde_json::from_slice(&body) {
        Ok(json) => json,
        Err(err) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                &format!("body isn't valid JSON: {err}"),
            );
        }
    };
    if let Err(reason) = sanitize::check_body(&json, caller.allows_local_files()) {
        return api_error(StatusCode::FORBIDDEN, "permission_error", &reason);
    }

    let model = json.get("model").and_then(Value::as_str).map(str::to_owned);
    let stream = json.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let id = app.log.start(RequestEntry::new(
        caller.label(),
        "POST",
        path,
        model,
        stream,
    ));
    let started = Instant::now();

    // GenieX runs one inference at a time: wait in line.
    app.enter_queue();
    let Ok(permit) = app.queue.clone().acquire_owned().await else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_error",
            "the gateway is shutting down",
        );
    };
    app.leave_queue_and_start();
    let exchange = Exchange {
        log: app.log.clone(),
        id,
        started,
        _turn: (permit, ActiveRequest(app.clone())),
    };

    let upstream = match app.services.server.ensure_running().await {
        Ok(url) => url,
        Err(err) => {
            return exchange.fail(
                StatusCode::SERVICE_UNAVAILABLE,
                &format!("GenieX couldn't start: {err}"),
            );
        }
    };
    let response = match app
        .http
        .post(format!("{upstream}{path}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return exchange.fail(
                StatusCode::BAD_GATEWAY,
                &format!("couldn't reach GenieX: {err}"),
            );
        }
    };

    if stream && response.status().is_success() {
        streamed(exchange, response)
    } else {
        buffered(exchange, response).await
    }
}

/// Non-streaming responses and errors: read fully, log, pass on.
async fn buffered(exchange: Exchange, response: reqwest::Response) -> Response {
    let status = response.status();
    let bytes = response.bytes().await.unwrap_or_default();
    let usage = serde_json::from_slice::<Value>(&bytes)
        .map(|json| Usage::from_json(&json))
        .unwrap_or_default();
    exchange.log.update(exchange.id, |entry| {
        entry.status = Some(status.as_u16());
        entry.duration_ms = Some(elapsed_ms(exchange.started));
        usage.apply(entry);
        if !status.is_success() {
            entry.error = Some(upstream_message(&bytes));
        }
    });
    if status.is_success() {
        ([(header::CONTENT_TYPE, "application/json")], bytes).into_response()
    } else {
        upstream_error(status, &bytes)
    }
}

/// Server-sent events: forward chunk by chunk, noting time to first token and
/// usage. The queue turn is held until the stream ends.
fn streamed(exchange: Exchange, response: reqwest::Response) -> Response {
    let status = response.status();
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(64);
    tokio::spawn(async move {
        let mut upstream = response.bytes_stream();
        let mut scanner = SseUsage::default();
        let mut first_token_ms = None;
        let mut error = None;
        while let Some(chunk) = upstream.next().await {
            match chunk {
                Ok(bytes) => {
                    first_token_ms.get_or_insert_with(|| elapsed_ms(exchange.started));
                    scanner.feed(&bytes);
                    if tx.send(Ok(bytes)).await.is_err() {
                        error = Some("client disconnected".to_owned());
                        break;
                    }
                }
                Err(err) => {
                    error = Some(format!("stream from GenieX broke: {err}"));
                    let _ = tx.send(Err(std::io::Error::other(err))).await;
                    break;
                }
            }
        }
        exchange.log.update(exchange.id, |entry| {
            entry.status = Some(status.as_u16());
            entry.duration_ms = Some(elapsed_ms(exchange.started));
            entry.first_token_ms = first_token_ms;
            scanner.usage.apply(entry);
            entry.error = error;
        });
    });

    (
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        Body::from_stream(ReceiverStream::new(rx)),
    )
        .into_response()
}

fn elapsed_ms(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_millis()).unwrap_or(u64::MAX)
}
