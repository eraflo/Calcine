//! Ollama's API under `/api`, for apps that only speak Ollama. See
//! [`crate::ollama`] for the translation.

use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Extension, Json, Router};
use calcine_core::models::{LocalModel, ModelType, Runtime};
use futures_util::StreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::caller::Caller;
use crate::error::upstream_message;
use crate::ollama::{self, Answer, Mode};
use crate::proxy::{self, Exchange, elapsed_ms};
use crate::state::AppState;
use crate::{context, structured};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/version", get(version))
        .route("/api/tags", get(tags))
        .route("/api/ps", get(running))
        .route("/api/show", post(show))
        .route("/api/chat", post(chat))
        .route("/api/generate", post(generate))
        .route("/api/{*rest}", any(unsupported))
}

/// What Ollama answers on `/`; clients check it to find a server.
pub async fn root() -> &'static str {
    "Ollama is running"
}

async fn version() -> Json<Value> {
    Json(json!({ "version": ollama::COMPAT_VERSION }))
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

async fn unsupported(uri: axum::http::Uri) -> Response {
    error(
        StatusCode::NOT_IMPLEMENTED,
        &format!(
            "Calcine doesn't support {}: download and remove models in the Calcine app",
            uri.path()
        ),
    )
}

async fn local_models(app: &AppState, caller: &Caller) -> Result<Vec<LocalModel>, Box<Response>> {
    if !caller.can_infer() {
        return Err(Box::new(error(
            StatusCode::FORBIDDEN,
            "this API key can't list models",
        )));
    }
    app.services
        .models
        .list()
        .await
        .map_err(|err| Box::new(error(StatusCode::SERVICE_UNAVAILABLE, &err.to_string())))
}

/// One entry per precision, named like GenieX ids: `name:precision`.
fn tag_entries(model: &LocalModel) -> Vec<Value> {
    let precisions: Vec<Option<&str>> = if model.precisions.is_empty() {
        vec![None]
    } else {
        model.precisions.iter().map(|p| Some(p.as_str())).collect()
    };
    precisions
        .into_iter()
        .map(|precision| {
            let name =
                precision.map_or_else(|| model.name.clone(), |p| format!("{}:{p}", model.name));
            json!({
                "name": name,
                "model": name,
                "modified_at": ollama::now_rfc3339(),
                "size": model.size_bytes,
                "digest": hex::encode(Sha256::digest(name.as_bytes())),
                "details": details(model, precision),
            })
        })
        .collect()
}

fn details(model: &LocalModel, precision: Option<&str>) -> Value {
    json!({
        "parent_model": "",
        "format": match model.runtime {
            Runtime::LlamaCpp => "gguf",
            Runtime::Qairt => "qairt",
            Runtime::Unknown => "unknown",
        },
        "family": model.name.split('/').next().unwrap_or(""),
        "families": Value::Null,
        "parameter_size": "",
        "quantization_level": precision.unwrap_or(""),
    })
}

async fn tags(State(app): State<Arc<AppState>>, Extension(caller): Extension<Caller>) -> Response {
    match local_models(&app, &caller).await {
        Ok(models) => {
            let entries: Vec<Value> = models.iter().flat_map(tag_entries).collect();
            Json(json!({ "models": entries })).into_response()
        }
        Err(response) => *response,
    }
}

/// GenieX doesn't say which model it holds in memory.
async fn running() -> Json<Value> {
    Json(json!({ "models": [] }))
}

async fn show(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    let Ok(body) = serde_json::from_slice::<Value>(&body) else {
        return error(StatusCode::BAD_REQUEST, "body isn't valid JSON");
    };
    let Some(name) = body
        .get("model")
        .or_else(|| body.get("name"))
        .and_then(Value::as_str)
        .map(ollama::model_id)
    else {
        return error(StatusCode::BAD_REQUEST, "model is required");
    };
    let models = match local_models(&app, &caller).await {
        Ok(models) => models,
        Err(response) => return *response,
    };
    let (base, precision) = match name.split_once(':') {
        Some((base, precision)) => (base, Some(precision)),
        None => (name.as_str(), None),
    };
    let Some(model) = models.iter().find(|model| {
        model.name == base && precision.is_none_or(|p| model.precisions.iter().any(|m| m == p))
    }) else {
        return error(StatusCode::NOT_FOUND, &format!("model '{name}' not found"));
    };
    let mut capabilities = vec!["completion"];
    if model.model_type == ModelType::Vlm {
        capabilities.push("vision");
    }
    Json(json!({
        "modelfile": "",
        "parameters": "",
        "template": "",
        "details": details(model, precision.or(model.precisions.first().map(String::as_str))),
        "model_info": {},
        "capabilities": capabilities,
        "modified_at": ollama::now_rfc3339(),
    }))
    .into_response()
}

async fn chat(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    answer(app, caller, Mode::Chat, &body).await
}

async fn generate(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    answer(app, caller, Mode::Generate, &body).await
}

async fn answer(app: Arc<AppState>, caller: Caller, mode: Mode, body: &[u8]) -> Response {
    let Ok(json) = serde_json::from_slice::<Value>(body) else {
        return error(StatusCode::BAD_REQUEST, "body isn't valid JSON");
    };
    if mode == Mode::Generate && ollama::is_load_only(&json) {
        return load(app, caller, &json).await;
    }
    let translated = match mode {
        Mode::Chat => ollama::chat_request(&json),
        Mode::Generate => ollama::generate_request(&json),
    };
    let mut translated = match translated {
        Ok(translated) => translated,
        Err(message) => return error(StatusCode::BAD_REQUEST, &message),
    };
    let path = match mode {
        Mode::Chat => "/api/chat",
        Mode::Generate => "/api/generate",
    };
    // Ollama forgets the oldest messages that don't fit: so does Calcine.
    if caller.can_infer()
        && let Err(message) = context::fit(&app, &mut translated.body).await
    {
        return error(StatusCode::BAD_REQUEST, &message);
    }
    // `format` (JSON or a schema): checked by Calcine, GenieX ignores it.
    if let Some(format) = structured::requested(&translated.body) {
        return structured_answer(&app, &caller, &translated, format, mode, path).await;
    }
    let upstream = Bytes::from(translated.body.to_string());
    let (exchange, response) = match proxy::send(
        &app,
        &caller,
        path,
        "/v1/chat/completions",
        &translated.body,
        upstream,
    )
    .await
    {
        Ok(sent) => sent,
        Err(refused) => return error(refused.status, &refused.message),
    };
    exchange
        .log
        .update(exchange.id, |entry| entry.stream = translated.stream);

    let status = response.status();
    if !status.is_success() {
        let bytes = response.bytes().await.unwrap_or_default();
        let message = upstream_message(&bytes);
        exchange.log_failure(status, &message);
        return error(status, &message);
    }
    let answer = Answer::new(&translated.model, mode);
    if translated.stream {
        stream(exchange, response, answer)
    } else {
        whole(exchange, response, answer).await
    }
}

/// A structured answer, as Ollama would give it: the JSON is the content.
async fn structured_answer(
    app: &Arc<AppState>,
    caller: &Caller,
    translated: &ollama::Translated,
    format: structured::Format,
    mode: Mode,
    path: &'static str,
) -> Response {
    let started = std::time::Instant::now();
    let completion = match structured::generate(
        app,
        caller,
        &translated.body,
        &structured::Goal::Json(format),
        path,
        translated.stream,
    )
    .await
    {
        Ok(completion) => completion,
        Err(failure) => return error(failure.status, &failure.message),
    };
    // Replay the completion as one event, so Ollama's shape is built once.
    let message = completion
        .pointer("/choices/0/message")
        .cloned()
        .unwrap_or_default();
    let event = json!({
        "choices": [{
            "delta": { "content": message["content"], "reasoning_content": message["reasoning_content"] },
            "finish_reason": "stop",
        }],
        "usage": completion["usage"],
        "timings": completion["timings"],
    });
    let mut answer = Answer::new(&translated.model, mode);
    let pieces = answer.feed(format!("data: {event}\n").as_bytes());
    let total_ms = elapsed_ms(started);
    if translated.stream {
        let mut body: Vec<u8> = Vec::new();
        for piece in pieces.iter().chain(std::iter::once(&answer.done(total_ms))) {
            body.extend_from_slice(&line(piece));
        }
        ([(header::CONTENT_TYPE, "application/x-ndjson")], body).into_response()
    } else {
        Json(answer.whole(total_ms)).into_response()
    }
}

/// `/api/generate` with no prompt loads the model (or, with `keep_alive: 0`,
/// asks to unload it, which GenieX does by itself after the unload delay).
async fn load(app: Arc<AppState>, caller: Caller, json: &Value) -> Response {
    let Some(model) = json
        .get("model")
        .and_then(Value::as_str)
        .map(ollama::model_id)
    else {
        return error(StatusCode::BAD_REQUEST, "model is required");
    };
    if json.get("keep_alive").and_then(Value::as_i64) == Some(0) {
        let mut reply = ollama::loaded(&model);
        reply["done_reason"] = json!("unload");
        return Json(reply).into_response();
    }
    let body = json!({
        "model": model,
        "messages": [{ "role": "user", "content": "Hi" }],
        "max_tokens": 1,
        "enable_think": false,
        "stream": false,
    });
    let upstream = Bytes::from(body.to_string());
    let (exchange, response) = match proxy::send(
        &app,
        &caller,
        "/api/generate",
        "/v1/chat/completions",
        &body,
        upstream,
    )
    .await
    {
        Ok(sent) => sent,
        Err(refused) => return error(refused.status, &refused.message),
    };
    let status = response.status();
    let bytes = response.bytes().await.unwrap_or_default();
    if !status.is_success() {
        let message = upstream_message(&bytes);
        exchange.log_failure(status, &message);
        return error(status, &message);
    }
    exchange.log.update(exchange.id, |entry| {
        entry.status = Some(status.as_u16());
        entry.duration_ms = Some(elapsed_ms(exchange.started));
    });
    Json(ollama::loaded(&model)).into_response()
}

/// JSON lines, one per piece of text, then a last line with statistics.
fn stream(exchange: Exchange, response: reqwest::Response, mut answer: Answer) -> Response {
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(64);
    tokio::spawn(async move {
        let mut upstream = response.bytes_stream();
        let mut first_token_ms = None;
        let mut broken = None;
        'read: while let Some(chunk) = upstream.next().await {
            let bytes = match chunk {
                Ok(bytes) => bytes,
                Err(err) => {
                    broken = Some(format!("stream from GenieX broke: {err}"));
                    break;
                }
            };
            for piece in answer.feed(&bytes) {
                first_token_ms.get_or_insert_with(|| elapsed_ms(exchange.started));
                if tx.send(Ok(line(&piece))).await.is_err() {
                    broken = Some("client disconnected".to_owned());
                    break 'read;
                }
            }
        }
        let error = broken.or_else(|| answer.error().map(str::to_owned));
        let last = match &error {
            Some(message) => json!({ "error": message }),
            None => answer.done(elapsed_ms(exchange.started)),
        };
        let _ = tx.send(Ok(line(&last))).await;
        finish_log(&exchange, &answer, first_token_ms, error);
    });
    (
        [(header::CONTENT_TYPE, "application/x-ndjson")],
        Body::from_stream(ReceiverStream::new(rx)),
    )
        .into_response()
}

/// The whole answer in one object (`"stream": false`).
async fn whole(exchange: Exchange, response: reqwest::Response, mut answer: Answer) -> Response {
    let mut upstream = response.bytes_stream();
    let mut first_token_ms = None;
    let mut broken = None;
    while let Some(chunk) = upstream.next().await {
        match chunk {
            Ok(bytes) => {
                if !answer.feed(&bytes).is_empty() {
                    first_token_ms.get_or_insert_with(|| elapsed_ms(exchange.started));
                }
            }
            Err(err) => {
                broken = Some(format!("stream from GenieX broke: {err}"));
                break;
            }
        }
    }
    let error_message = broken.or_else(|| answer.error().map(str::to_owned));
    finish_log(&exchange, &answer, first_token_ms, error_message.clone());
    match error_message {
        Some(message) => error(StatusCode::INTERNAL_SERVER_ERROR, &message),
        None => Json(answer.whole(elapsed_ms(exchange.started))).into_response(),
    }
}

fn finish_log(
    exchange: &Exchange,
    answer: &Answer,
    first_token_ms: Option<u64>,
    error: Option<String>,
) {
    exchange.log.update(exchange.id, |entry| {
        entry.status = Some(200);
        entry.duration_ms = Some(elapsed_ms(exchange.started));
        entry.first_token_ms = first_token_ms;
        entry.prompt_tokens = answer.prompt_tokens();
        entry.completion_tokens = answer.completion_tokens();
        entry.tokens_per_second = answer.tokens_per_second();
        entry.error = error;
    });
}

fn line(value: &Value) -> Bytes {
    let mut bytes = value.to_string().into_bytes();
    bytes.push(b'\n');
    Bytes::from(bytes)
}
