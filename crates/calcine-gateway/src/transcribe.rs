//! `POST /v1/audio/transcriptions`, as in OpenAI's API, answered by a chat
//! model that listens (Gemma 4, Gemma 3n): GenieX has no transcription
//! models yet.
//!
//! Only speech is sent, in pieces of at most 28 s (see [`crate::audio`]),
//! each with an instruction to transcribe it word for word. The pieces share
//! one log entry and one turn in the queue. The audio is never stored.

use std::fmt::Write as _;
use std::sync::Arc;

use axum::Extension;
use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use calcine_core::models::Runtime;
use serde_json::{Value, json};

use crate::audio;
use crate::caller::Caller;
use crate::error::api_error;
use crate::proxy::{self, elapsed_ms};
use crate::state::AppState;
use crate::structured;

const LOGGED_PATH: &str = "/v1/audio/transcriptions";

/// What the client asked for.
#[derive(Debug, Default)]
struct Request {
    file: Option<Bytes>,
    model: Option<String>,
    language: Option<String>,
    prompt: Option<String>,
    text_only: bool,
    /// Calcine's own field: `gpu`, `cpu` or `npu`.
    compute: Option<String>,
}

pub async fn transcriptions(
    State(app): State<Arc<AppState>>,
    Extension(caller): Extension<Caller>,
    multipart: Multipart,
) -> Response {
    if !caller.can_infer() {
        return api_error(
            StatusCode::FORBIDDEN,
            "permission_error",
            "this API key can't run models",
        );
    }
    let request = match read(multipart).await {
        Ok(request) => request,
        Err(message) => return invalid(&message),
    };
    let Some(file) = request.file.as_ref() else {
        return invalid("send the audio as a `file` field (multipart/form-data)");
    };
    let Some(model) = request
        .model
        .clone()
        .filter(|model| !model.trim().is_empty())
    else {
        return invalid("choose a `model` that understands speech, e.g. Gemma 4 E4B");
    };
    let audio = match audio::decode_wav(file) {
        Ok(audio) => audio,
        Err(message) => return invalid(&message),
    };
    // Nobody speaks: answer without asking a model, which would invent words.
    let Some(spoken) = audio::speech(&audio) else {
        return answer(&request, "");
    };
    let pieces = audio::pieces(spoken);
    if pieces.is_empty() {
        return answer(&request, "");
    }
    let unit = compute(&app, &model, request.compute.as_deref()).await;

    let mut texts = Vec::new();
    let mut usage = (0_u64, 0_u64);
    let mut exchange = None;
    for piece in pieces {
        let body = chat_request(&model, piece, &request, &unit);
        // The first piece takes the turn in the queue and the log entry; the
        // next ones follow while it's held.
        let reply = if exchange.is_some() {
            structured::retry(&app, &body).await
        } else {
            let bytes = Bytes::from(body.to_string());
            match proxy::send(
                &app,
                &caller,
                LOGGED_PATH,
                "/v1/chat/completions",
                &body,
                bytes,
            )
            .await
            {
                Ok((sent, response)) => {
                    exchange = Some(sent);
                    Ok(response)
                }
                Err(refused) => return refused.into_response(),
            }
        };
        let completion = match structured::read_completion(reply).await {
            Ok(completion) => completion,
            Err((status, message)) => {
                if let Some(exchange) = &exchange {
                    exchange.log_failure(status, &message);
                }
                return api_error(status, "server_error", &message);
            }
        };
        usage.0 += structured::token_count(&completion, "prompt_tokens");
        usage.1 += structured::token_count(&completion, "completion_tokens");
        let text = completion
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(clean)
            .unwrap_or_default();
        if !text.is_empty() {
            texts.push(text);
        }
    }
    if let Some(exchange) = &exchange {
        exchange.log.update(exchange.id, |entry| {
            entry.status = Some(200);
            entry.duration_ms = Some(elapsed_ms(exchange.started));
            entry.prompt_tokens = Some(usage.0);
            entry.completion_tokens = Some(usage.1);
        });
    }
    answer(&request, &texts.join(" "))
}

async fn read(mut multipart: Multipart) -> Result<Request, String> {
    let mut request = Request::default();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| format!("couldn't read the form: {err}"))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "file" {
            request.file = Some(
                field
                    .bytes()
                    .await
                    .map_err(|err| format!("couldn't read the audio: {err}"))?,
            );
            continue;
        }
        let value = field
            .text()
            .await
            .map_err(|err| format!("couldn't read `{name}`: {err}"))?;
        let value = value.trim().to_owned();
        match name.as_str() {
            "model" => request.model = Some(value),
            "language" if !value.is_empty() => request.language = Some(value),
            "prompt" if !value.is_empty() => request.prompt = Some(value),
            "compute" if !value.is_empty() => request.compute = Some(value.to_ascii_lowercase()),
            "response_format" => match value.as_str() {
                "" | "json" => request.text_only = false,
                "text" => request.text_only = true,
                other => {
                    return Err(format!(
                        "response_format {other} isn't supported: use json or text"
                    ));
                }
            },
            // OpenAI's other fields (temperature, timestamp granularities)
            // don't apply: transcription is always deterministic.
            _ => {}
        }
    }
    Ok(request)
}

/// Where the model runs. Speech encoders hang on the NPU through llama.cpp
/// (GenieX 0.10), so llama.cpp models listen on the GPU by default, or on
/// the CPU while the rest runs on the NPU.
async fn compute(app: &AppState, model: &str, asked: Option<&str>) -> Value {
    let name = model.split_once(':').map_or(model, |(name, _)| name);
    let runtime = app
        .services
        .models
        .list()
        .await
        .ok()
        .and_then(|models| models.into_iter().find(|local| local.name == name))
        .map(|local| local.runtime);
    if runtime != Some(Runtime::LlamaCpp) {
        return json!({});
    }
    match asked {
        Some("npu") => json!({ "compute": "npu", "vit_compute": "CPU" }),
        Some("cpu") => json!({ "compute": "cpu" }),
        Some("gpu") => json!({ "compute": "gpu" }),
        _ if cfg!(windows) => json!({ "compute": "gpu" }),
        _ => json!({ "compute": "cpu" }),
    }
}

/// The chat request for one piece of audio.
fn chat_request(model: &str, piece: &[f32], request: &Request, unit: &Value) -> Value {
    let wav = BASE64.encode(audio::encode_wav(piece));
    let language = request.language.as_deref().map_or_else(
        || ", in the language it is spoken in".to_owned(),
        |code| format!(", in {}", language_name(code)),
    );
    let mut instruction = format!(
        "Transcribe this audio verbatim{language}. Output only the transcription, nothing else."
    );
    if let Some(prompt) = &request.prompt {
        let _ = write!(instruction, " Names and terms it may contain: {prompt}");
    }
    // About 4 tokens a second of speech, with room to spare.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let max_tokens = 64 + (piece.len() / audio::SAMPLE_RATE as usize) as u64 * 10;
    let mut body = json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "input_audio", "input_audio": { "data": wav, "format": "wav" } },
                { "type": "text", "text": instruction },
            ],
        }],
        "temperature": 0,
        "max_tokens": max_tokens,
        "enable_think": false,
        "reasoning_format": "deepseek",
        "stream": false,
    });
    if let (Some(body), Some(unit)) = (body.as_object_mut(), unit.as_object()) {
        body.extend(unit.clone());
    }
    body
}

/// ISO 639-1 codes the instruction names in full; others are passed as is.
fn language_name(code: &str) -> &str {
    match code.to_ascii_lowercase().as_str() {
        "fr" => "French",
        "en" => "English",
        "es" => "Spanish",
        "de" => "German",
        "it" => "Italian",
        "pt" => "Portuguese",
        "nl" => "Dutch",
        "pl" => "Polish",
        "ru" => "Russian",
        "ar" => "Arabic",
        "hi" => "Hindi",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        _ => code,
    }
}

/// The transcription alone: models sometimes add a label or quotes.
fn clean(text: &str) -> String {
    let mut text = text.trim();
    for label in ["Transcription:", "Transcript:", "Transcription :"] {
        if let Some(rest) = text.strip_prefix(label) {
            text = rest.trim();
        }
    }
    let quoted = text.len() >= 2
        && ((text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('«') && text.ends_with('»')));
    if quoted {
        text = text
            .trim_start_matches(['"', '«'])
            .trim_end_matches(['"', '»'])
            .trim();
    }
    text.to_owned()
}

fn answer(request: &Request, text: &str) -> Response {
    if request.text_only {
        (
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            text.to_owned(),
        )
            .into_response()
    } else {
        axum::Json(json!({ "text": text })).into_response()
    }
}

fn invalid(message: &str) -> Response {
    api_error(StatusCode::BAD_REQUEST, "invalid_request_error", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_transcription() {
        assert_eq!(clean("  Bonjour à tous.\n"), "Bonjour à tous.");
        assert_eq!(clean("Transcription: Hello there."), "Hello there.");
        assert_eq!(clean("\"Hello there.\""), "Hello there.");
        assert_eq!(clean("« Bonjour »"), "Bonjour");
        // Quotes inside the text stay.
        assert_eq!(clean("He said \"hi\" twice"), "He said \"hi\" twice");
    }

    #[test]
    fn asks_for_the_language_when_given() {
        let piece = vec![0.1_f32; audio::SAMPLE_RATE as usize];
        let request = Request {
            language: Some("fr".into()),
            prompt: Some("Calcine, GenieX".into()),
            ..Request::default()
        };
        let body = chat_request("m", &piece, &request, &json!({ "compute": "gpu" }));
        let instruction = body
            .pointer("/messages/0/content/1/text")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(instruction.contains("verbatim, in French."));
        assert!(instruction.contains("Calcine, GenieX"));
        assert_eq!(body["compute"], "gpu");
        assert_eq!(body["temperature"], 0);
        assert_eq!(
            body.pointer("/messages/0/content/0/type").unwrap(),
            "input_audio"
        );

        let any = chat_request("m", &piece, &Request::default(), &json!({}));
        let instruction = any
            .pointer("/messages/0/content/1/text")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(instruction.contains("in the language it is spoken in"));
        assert!(any.get("compute").is_none());
    }
}
