//! Fitting a conversation into the model's context window.
//!
//! GenieX refuses a prompt longer than the window (4096 tokens for most AI
//! Hub models). With `"truncation": "auto"` (OpenAI's name for it), and
//! always for Ollama clients as Ollama does, Calcine forgets the oldest
//! exchanges instead, keeping the system prompt and room for the reply.

use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use calcine_core::context::{forget_oldest, reply_reserve};
use serde_json::{Value, json};

use crate::state::AppState;

/// How many messages were left out to fit, on the response.
pub const FORGOTTEN_HEADER: &str = "x-calcine-forgotten-messages";

/// Whether the request asks to forget what doesn't fit. Removes the field,
/// which GenieX doesn't know.
pub fn take_truncation(request: &mut Value) -> bool {
    request
        .as_object_mut()
        .and_then(|object| object.remove("truncation"))
        .is_some_and(|truncation| truncation == "auto")
}

/// Forget the oldest messages that don't fit: how many were forgotten, or
/// why the conversation can't fit. Counting problems (an unknown model, an
/// unreadable tokenizer) leave the request as it is, for GenieX to judge.
pub async fn fit(app: &AppState, request: &mut Value) -> Result<usize, String> {
    let Some(model) = request
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return Ok(0);
    };
    let services = &app.services;
    let (Ok(window), Ok(count)) = (
        services.context_window(&model).await,
        services.context.count(&model, request).await,
    ) else {
        return Ok(0);
    };
    let reserve = reply_reserve(request, window);
    let Some(messages) = request.get_mut("messages").and_then(Value::as_array_mut) else {
        return Ok(0);
    };
    forget_oldest(messages, &count, window.saturating_sub(reserve)).ok_or_else(|| {
        format!(
            "the last message doesn't fit in {model}'s context window of {window} tokens, with \
             {reserve} kept for the reply: shorten it, or ask for a shorter reply with max_tokens"
        )
    })
}

/// Say on the response how many messages were forgotten.
pub fn mark(mut response: Response, forgotten: usize) -> Response {
    if forgotten > 0 {
        response
            .headers_mut()
            .insert(FORGOTTEN_HEADER, HeaderValue::from(forgotten));
    }
    response
}

/// The prompt doesn't fit, in the OpenAI shape.
pub fn too_long(message: &str) -> Response {
    use axum::Json;
    use axum::response::IntoResponse;
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": {
            "message": message,
            "type": "invalid_request_error",
            "code": "context_length_exceeded",
        } })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_truncation_field() {
        let mut request = json!({ "model": "m", "truncation": "auto" });
        assert!(take_truncation(&mut request));
        assert_eq!(request, json!({ "model": "m" }));
        let mut request = json!({ "truncation": "disabled" });
        assert!(!take_truncation(&mut request));
        assert!(!take_truncation(&mut json!({})));
    }
}
