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
use crate::error::{api_error, is_context_error, upstream_error, upstream_message};
use crate::log::{RequestEntry, RequestLog, SseUsage, Usage};
use crate::security::sanitize;
use crate::state::{ActiveRequest, AppState};

/// One request in flight: where it's logged, when it started, and its turn
/// in the queue (released when this is dropped).
pub(crate) struct Exchange {
    pub log: Arc<RequestLog>,
    pub id: u32,
    pub started: Instant,
    _turn: (OwnedSemaphorePermit, ActiveRequest),
}

impl Exchange {
    /// Log a failure. The caller answers in its own error format.
    pub fn log_failure(&self, status: StatusCode, message: &str) {
        self.log.update(self.id, |entry| {
            entry.status = Some(status.as_u16());
            entry.duration_ms = Some(elapsed_ms(self.started));
            entry.error = Some(message.to_owned());
        });
    }
}

/// Why a request didn't reach GenieX.
#[derive(Debug)]
pub(crate) struct Refused {
    pub status: StatusCode,
    pub kind: &'static str,
    pub message: String,
}

impl Refused {
    fn new(status: StatusCode, kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            kind,
            message: message.into(),
        }
    }

    /// In the OpenAI error format.
    pub fn into_response(self) -> Response {
        api_error(self.status, self.kind, &self.message)
    }
}

pub async fn inference(
    app: Arc<AppState>,
    caller: Caller,
    path: &'static str,
    body: Bytes,
) -> Response {
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
    let stream = json.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let (exchange, response) = match send(&app, &caller, path, path, &json, body).await {
        Ok(sent) => sent,
        Err(refused) => return refused.into_response(),
    };
    if stream && response.status().is_success() {
        streamed(exchange, response)
    } else {
        buffered(exchange, response).await
    }
}

/// Check, log and queue a request, then send `body` (the JSON `json`) to
/// GenieX at `upstream_path`. The request log shows `logged_path`.
pub(crate) async fn send(
    app: &Arc<AppState>,
    caller: &Caller,
    logged_path: &'static str,
    upstream_path: &'static str,
    json: &Value,
    body: Bytes,
) -> Result<(Exchange, reqwest::Response), Refused> {
    if !caller.can_infer() {
        return Err(Refused::new(
            StatusCode::FORBIDDEN,
            "permission_error",
            "this API key can't run models",
        ));
    }
    if let Err(reason) = sanitize::check_body(json, caller.allows_local_files()) {
        return Err(Refused::new(
            StatusCode::FORBIDDEN,
            "permission_error",
            reason,
        ));
    }

    let model = json.get("model").and_then(Value::as_str).map(str::to_owned);
    let stream = json.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let id = app.log.start(RequestEntry::new(
        caller.label(),
        "POST",
        logged_path,
        model,
        stream,
    ));
    let started = Instant::now();

    // GenieX runs one inference at a time: wait in line.
    app.enter_queue();
    let Ok(permit) = app.queue.clone().acquire_owned().await else {
        return Err(Refused::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_error",
            "the gateway is shutting down",
        ));
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
            let message = format!("GenieX couldn't start: {err}");
            exchange.log_failure(StatusCode::SERVICE_UNAVAILABLE, &message);
            return Err(Refused::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "server_error",
                message,
            ));
        }
    };
    let post = |body: Bytes| {
        app.http
            .post(format!("{upstream}{upstream_path}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
    };
    match post(body.clone()).await {
        Ok(response) if response.status() == StatusCode::BAD_REQUEST => {
            let bytes = response.bytes().await.unwrap_or_default();
            if !is_context_error(&bytes) {
                return Ok((exchange, rebuilt(StatusCode::BAD_REQUEST, bytes)));
            }
            // GenieX 0.8 counts the conversation still in its cache against
            // the window when the new one doesn't continue it (a regenerated
            // reply, another client): a prompt that fits is refused. A
            // one-token request in between empties the cache.
            clear_cache(app, &upstream, json).await;
            match post(body).await {
                Ok(response) => Ok((exchange, response)),
                Err(_) => Ok((exchange, rebuilt(StatusCode::BAD_REQUEST, bytes))),
            }
        }
        Ok(response) => Ok((exchange, response)),
        Err(err) => {
            let message = format!("couldn't reach GenieX: {err}");
            exchange.log_failure(StatusCode::BAD_GATEWAY, &message);
            Err(Refused::new(
                StatusCode::BAD_GATEWAY,
                "server_error",
                message,
            ))
        }
    }
}

/// A response already read, to pass on as if it hadn't been.
fn rebuilt(status: StatusCode, body: Bytes) -> reqwest::Response {
    let mut response = axum::http::Response::new(body);
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    reqwest::Response::from(response)
}

/// Run a one-token request so GenieX drops the conversation it caches.
async fn clear_cache(app: &AppState, upstream: &str, request: &Value) {
    let Some(model) = request.get("model") else {
        return;
    };
    let reset = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": "." }],
        "max_tokens": 1,
        "enable_think": false,
    });
    let _ = app
        .http
        .post(format!("{upstream}/v1/chat/completions"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(reset.to_string())
        .send()
        .await;
    tracing::info!("cleared GenieX's cached conversation to retry a prompt it refused");
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
            entry.error = error.or(scanner.error);
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

pub(crate) fn elapsed_ms(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_millis()).unwrap_or(u64::MAX)
}
