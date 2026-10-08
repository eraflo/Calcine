//! Errors in the OpenAI shape, so client libraries surface the message.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

pub fn api_error(status: StatusCode, kind: &str, message: &str) -> Response {
    (
        status,
        Json(json!({ "error": { "message": message, "type": kind, "code": Value::Null } })),
    )
        .into_response()
}

/// GenieX answers `{"error": "…"}`; rewrap it, keeping the status code.
pub fn upstream_error(status: StatusCode, body: &[u8]) -> Response {
    let message = upstream_message(body);
    let kind = if status.is_client_error() {
        "invalid_request_error"
    } else {
        "server_error"
    };
    api_error(status, kind, &message)
}

pub fn upstream_message(body: &[u8]) -> String {
    serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|value| match value.get("error") {
            Some(Value::String(message)) => Some(message.clone()),
            Some(error) => error
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_owned),
            None => None,
        })
        .unwrap_or_else(|| String::from_utf8_lossy(body).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_genie_errors() {
        let body = br#"{"error":"SDKError(File not found), model 'x' not found in local cache"}"#;
        assert_eq!(
            upstream_message(body),
            "SDKError(File not found), model 'x' not found in local cache"
        );
    }

    #[test]
    fn keeps_openai_errors_and_plain_text() {
        assert_eq!(upstream_message(br#"{"error":{"message":"bad"}}"#), "bad");
        assert_eq!(upstream_message(b" oops \n"), "oops");
    }
}
