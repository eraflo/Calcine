//! A fake OpenAI-compatible inference server, standing in for `geniex serve`.
//!
//! Streams a canned reply word by word (with a short reasoning preamble unless
//! `enable_think` is false), so the gateway and the Chat page work anywhere.
//! Given tools, it calls the first one.

use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use axum::Json;
use axum::body::{Body, Bytes};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use calcine_core::Result;
use calcine_core::runtime::{InferenceServer, ServerOptions, ServerState};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_stream::wrappers::ReceiverStream;

const TOKEN_DELAY: Duration = Duration::from_millis(25);
/// Like AI Hub models: GenieX refuses longer prompts.
const WINDOW: usize = 4096;

#[derive(Debug)]
pub struct MockServer {
    running: tokio::sync::Mutex<Option<(String, oneshot::Sender<()>)>>,
    state: watch::Sender<ServerState>,
    options: Mutex<ServerOptions>,
}

impl Default for MockServer {
    fn default() -> Self {
        Self {
            running: tokio::sync::Mutex::new(None),
            state: watch::Sender::new(ServerState::Stopped),
            options: Mutex::new(ServerOptions::default()),
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

    fn options(&self) -> ServerOptions {
        *self.options.lock().unwrap_or_else(PoisonError::into_inner)
    }

    async fn set_options(&self, options: ServerOptions) -> Result<()> {
        options
            .validate()
            .map_err(calcine_core::Error::InvalidInput)?;
        *self.options.lock().unwrap_or_else(PoisonError::into_inner) = options;
        Ok(())
    }

    /// A canned completion whose speed and energy follow the power mode,
    /// returned within 0.4 s.
    async fn complete(&self, request: &Value) -> Result<Value> {
        let tokens = request["max_tokens"]
            .as_u64()
            .and_then(|tokens| u32::try_from(tokens).ok())
            .unwrap_or(16)
            .min(512);
        let speed = mode_speed(request["power_mode"].as_str().unwrap_or("burst"));
        let seconds = (f64::from(tokens) / speed).min(0.4);
        tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
        add_generation_joules(
            mode_watts(request["power_mode"].as_str().unwrap_or("burst")) * seconds,
        );
        Ok(json!({
            "object": "chat.completion",
            "choices": [{ "index": 0, "finish_reason": "length", "message": { "role": "assistant", "content": "Once upon a time…" } }],
            "usage": { "prompt_tokens": 48, "completion_tokens": tokens, "total_tokens": 48 + tokens },
            "timings": { "prompt_ms": 60.0, "prompt_per_second": 800.0, "predicted_per_second": speed },
        }))
    }
}

async fn chat_completions(Json(request): Json<Value>) -> Response {
    let model = request["model"].as_str().unwrap_or("mock").to_owned();
    if prompt_size(&request) > WINDOW {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": {
                "code": "context_length_exceeded",
                "message": "prompt is longer than the model's context window",
                "type": "invalid_request_error",
            } })),
        )
            .into_response();
    }
    if let Some(call) = tool_call(&request) {
        return call_tool(&model, &call, request["stream"].as_bool() == Some(true));
    }
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
    // Asked for JSON (Calcine's structured output): answer in JSON. A forced
    // tool call is the example the instruction shows, filled in.
    let system = request["messages"][0]["content"].as_str().unwrap_or("");
    let wants_json = system.contains("single valid JSON value");
    let example = system
        .split_once("with a real value:\n")
        .and_then(|(_, rest)| rest.lines().next())
        .and_then(|line| serde_json::from_str::<Value>(line).ok());
    let reply =
        if let Some(mut call) = example.filter(|_| system.contains("Reply with only the call")) {
            fill_in(&mut call);
            call.to_string()
        } else if wants_json {
            json!({ "reply": "Mock answer", "said": prompt }).to_string()
        } else {
            format!(
                "This is **Calcine's mock backend**: no model ran. You said: \"{prompt}\". Run \
             Calcine with GenieX on a Snapdragon device to talk to a real model."
            )
        };

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

/// Tokens the prompt takes, counted as the mock's context meter does.
fn prompt_size(request: &Value) -> usize {
    let messages = request["messages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    20 + messages
        .iter()
        .map(|message| message["content"].to_string().len().div_ceil(3) + 5)
        .sum::<usize>()
}

/// With tools and a user message last, the mock calls the first tool, with
/// placeholder arguments.
fn tool_call(request: &Value) -> Option<Value> {
    let tool = request["tools"].as_array()?.first()?;
    let last = request["messages"].as_array()?.last()?;
    if last["role"] != "user" {
        return None;
    }
    let name = tool.pointer("/function/name")?.as_str()?;
    let arguments: serde_json::Map<String, Value> = tool
        .pointer("/function/parameters/properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(key, property)| {
                    let value = match property["type"].as_str() {
                        Some("integer" | "number") => json!(1),
                        Some("boolean") => json!(true),
                        Some("array") => json!([]),
                        Some("object") => json!({}),
                        _ => json!("mock"),
                    };
                    (key.clone(), value)
                })
                .collect()
        })
        .unwrap_or_default();
    Some(json!({
        "id": "call_mock",
        "type": "function",
        "function": { "name": name, "arguments": Value::Object(arguments).to_string() },
    }))
}

/// Replace the example's `<placeholders>` with "mock".
fn fill_in(value: &mut Value) {
    match value {
        Value::String(text) if text.starts_with('<') => *text = "mock".into(),
        Value::Array(items) => items.iter_mut().for_each(fill_in),
        Value::Object(fields) => fields.values_mut().for_each(fill_in),
        _ => {}
    }
}

fn call_tool(model: &str, call: &Value, stream: bool) -> Response {
    if !stream {
        return Json(json!({
            "object": "chat.completion",
            "model": model,
            "choices": [{
                "index": 0,
                "finish_reason": "tool_calls",
                "message": { "role": "assistant", "content": "", "tool_calls": [call] },
            }],
            "usage": usage("call"),
        }))
        .into_response();
    }
    let mut call = call.clone();
    call["index"] = json!(0);
    let chunks = [
        json!({ "object": "chat.completion.chunk", "choices": [{ "index": 0, "delta": { "tool_calls": [call] }, "finish_reason": null }] }),
        json!({ "object": "chat.completion.chunk", "choices": [{ "index": 0, "delta": {}, "finish_reason": "tool_calls" }] }),
    ];
    let mut body: Vec<u8> = chunks
        .iter()
        .flat_map(|chunk| sse(chunk).to_vec())
        .collect();
    body.extend_from_slice(b"data:[DONE]\n\n");
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

/// Energy the mock's generations used, in millijoules, for the mock meter.
static GENERATION_MILLIJOULES: AtomicU64 = AtomicU64::new(0);

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn add_generation_joules(joules: f64) {
    GENERATION_MILLIJOULES.fetch_add((joules * 1000.0) as u64, Ordering::Relaxed);
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn generation_joules() -> f64 {
    GENERATION_MILLIJOULES.load(Ordering::Relaxed) as f64 / 1000.0
}

/// What generating adds to the idle draw, in watts, per power mode.
fn mode_watts(mode: &str) -> f64 {
    match mode {
        "burst" | "sustained_high_performance" => 7.0,
        "high_performance" => 6.0,
        "balanced" | "low_balanced" => 4.0,
        "high_power_saver" => 3.0,
        _ => 2.0,
    }
}

/// Plausible decode speeds per power mode, tokens per second.
pub(crate) fn mode_speed(mode: &str) -> f64 {
    match mode {
        "burst" | "sustained_high_performance" => 52.0,
        "high_performance" => 48.0,
        "balanced" | "low_balanced" => 38.0,
        "high_power_saver" => 30.0,
        _ => 22.0,
    }
}

fn sse(value: &Value) -> Bytes {
    Bytes::from(format!("data:{value}\n\n"))
}

fn usage(reply: &str) -> Value {
    let words = reply.split_whitespace().count();
    json!({ "prompt_tokens": 20, "completion_tokens": words, "total_tokens": 20 + words })
}
