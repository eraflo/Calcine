//! A fake OpenAI-compatible inference server, standing in for `geniex serve`.
//!
//! Streams a canned reply word by word (with a short reasoning preamble unless
//! `enable_think` is false), so the gateway and the Chat page work anywhere.

use std::convert::Infallible;
use std::time::Duration;

use async_trait::async_trait;
use axum::Json;
use axum::body::{Body, Bytes};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use calcine_core::Result;
use calcine_core::runtime::{InferenceServer, ServerState};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_stream::wrappers::ReceiverStream;

const TOKEN_DELAY: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub struct MockServer {
    running: tokio::sync::Mutex<Option<(String, oneshot::Sender<()>)>>,
    state: watch::Sender<ServerState>,
}

impl Default for MockServer {
    fn default() -> Self {
        Self {
            running: tokio::sync::Mutex::new(None),
            state: watch::Sender::new(ServerState::Stopped),
        }
    }
}

#[async_trait]
impl InferenceServer for MockServer {
    async fn ensure_running(&self) -> Result<String> {
        let mut running = self.running.lock().await;
        if let Some((url, _)) = running.as_ref() {
            return Ok(url.clone());
        }
        self.state.send_replace(ServerState::Starting);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}", listener.local_addr()?);
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let app = axum::Router::new()
            .route("/v1/", get(|| async { "Calcine mock inference server" }))
            .route("/v1/chat/completions", post(chat_completions));
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = stop_rx.await;
                })
                .await;
        });
        self.state.send_replace(ServerState::Ready {
            url: url.clone(),
            started_at_ms: 0,
        });
        *running = Some((url.clone(), stop_tx));
        Ok(url)
    }

    async fn stop(&self) -> Result<()> {
        if let Some((_, stop)) = self.running.lock().await.take() {
            let _ = stop.send(());
        }
        self.state.send_replace(ServerState::Stopped);
        Ok(())
    }

    fn state(&self) -> watch::Receiver<ServerState> {
        self.state.subscribe()
    }

    fn logs(&self) -> Vec<String> {
        vec!["Mock inference server: replies are canned, no model runs.".into()]
    }
}

async fn chat_completions(Json(request): Json<Value>) -> Response {
    let model = request["model"].as_str().unwrap_or("mock").to_owned();
    let prompt = request["messages"]
        .as_array()
        .and_then(|messages| messages.iter().rev().find(|m| m["role"] == "user"))
        .and_then(|message| message["content"].as_str())
        .unwrap_or("")
        .to_owned();
    let think = request["enable_think"].as_bool().unwrap_or(true);
    let reasoning = if think {
        "The user wants a demo reply. I'll answer briefly and mention the mock backend."
    } else {
        ""
    };
    let reply = format!(
        "This is **Calcine's mock backend**: no model ran. You said: \"{prompt}\". Run Calcine \
         with GenieX on a Snapdragon device to talk to a real model."
    );

    if request["stream"].as_bool() != Some(true) {
        return Json(json!({
            "object": "chat.completion",
            "model": model,
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": { "role": "assistant", "content": reply, "reasoning_content": reasoning },
            }],
            "usage": usage(&reply),
        }))
        .into_response();
    }

    let (tx, rx) = mpsc::channel::<std::result::Result<Bytes, Infallible>>(32);
    tokio::spawn(async move {
        for (field, text) in [
            ("reasoning_content", reasoning),
            ("content", reply.as_str()),
        ] {
            for word in text.split_inclusive(' ') {
                let chunk = json!({
                    "object": "chat.completion.chunk",
                    "choices": [{ "index": 0, "delta": { "role": "assistant", field: word }, "finish_reason": null }],
                });
                if tx.send(Ok(sse(&chunk))).await.is_err() {
                    return;
                }
                tokio::time::sleep(TOKEN_DELAY).await;
            }
        }
        let done = json!({ "object": "chat.completion.chunk", "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }] });
        let stats = json!({ "object": "chat.completion.chunk", "choices": [], "usage": usage(&reply), "timings": { "prompt_ms": 30.0, "predicted_per_second": 40.0 } });
        let _ = tx.send(Ok(sse(&done))).await;
        let _ = tx.send(Ok(sse(&stats))).await;
        let _ = tx.send(Ok(Bytes::from_static(b"data:[DONE]\n\n"))).await;
    });

    (
        [(header::CONTENT_TYPE, "text/event-stream")],
        Body::from_stream(ReceiverStream::new(rx)),
    )
        .into_response()
}

fn sse(value: &Value) -> Bytes {
    Bytes::from(format!("data:{value}\n\n"))
}

fn usage(reply: &str) -> Value {
    let words = reply.split_whitespace().count();
    json!({ "prompt_tokens": 20, "completion_tokens": words, "total_tokens": 20 + words })
}
