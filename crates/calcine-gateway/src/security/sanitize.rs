//! GenieX accepts local file paths and URLs inside request bodies (images,
//! grammars, draft models) and reads or fetches them itself. Unless a key is
//! allowed to, only inline `data:` images get through.
//!
//! Audio needs no check: GenieX always decodes `input_audio.data` as base64
//! (adding a `data:audio/<format>;base64,` prefix when missing), so it can't
//! point at a file or URL.

use serde_json::Value;

/// Why a body was refused.
pub type Refusal = String;

/// Check an inference request body for file paths and URLs.
pub fn check_body(body: &Value, allow_local_files: bool) -> Result<(), Refusal> {
    if allow_local_files {
        return Ok(());
    }

    if body
        .get("grammar_path")
        .and_then(Value::as_str)
        .is_some_and(|path| !path.is_empty())
    {
        return Err(
            "`grammar_path` reads a local file. Send `grammar_string` instead, or use \
                    a key allowed to read local files."
                .into(),
        );
    }

    if let Some(draft) = body.get("spec_draft_model").and_then(Value::as_str)
        && looks_like_path(draft)
    {
        return Err(
            "`spec_draft_model` must be a model name, not a file path, for this key.".into(),
        );
    }

    for message in body
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for part in message
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let image = part
                .pointer("/image_url/url")
                .or_else(|| part.get("image_url"))
                .and_then(Value::as_str);
            if let Some(image) = image
                && !is_inline(image)
            {
                return Err(
                    "images must be sent inline as `data:` URLs. This key can't make \
                            GenieX read local files or fetch URLs."
                        .into(),
                );
            }
        }
    }
    Ok(())
}

fn is_inline(source: &str) -> bool {
    source
        .trim_start()
        .get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
}

/// File paths (absolute, relative, `file:`) and GGUF files, as opposed to
/// catalog names like `org/model`.
fn looks_like_path(value: &str) -> bool {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    std::path::Path::new(value)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
        || lower.starts_with("file:")
        || value.starts_with(['/', '\\', '.', '~'])
        || value.contains('\\')
        || value.as_bytes().get(1) == Some(&b':')
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn chat_with(part: &Value) -> Value {
        json!({ "model": "m", "messages": [{ "role": "user", "content": [part] }] })
    }

    #[test]
    fn text_only_requests_pass() {
        let body = json!({ "model": "m", "messages": [{ "role": "user", "content": "hi" }] });
        assert_eq!(check_body(&body, false), Ok(()));
    }

    #[test]
    fn inline_images_and_audio_pass() {
        let image = chat_with(
            &json!({ "type": "image_url", "image_url": { "url": "data:image/png;base64,AAAA" } }),
        );
        assert_eq!(check_body(&image, false), Ok(()));
        // OpenAI's shape (raw base64 plus a format) and data URLs.
        for data in ["UklGRiQAAABXQVZF", "DATA:audio/wav;base64,AA"] {
            let audio = chat_with(
                &json!({ "type": "input_audio", "input_audio": { "data": data, "format": "wav" } }),
            );
            assert_eq!(check_body(&audio, false), Ok(()));
        }
    }

    #[test]
    fn local_paths_and_urls_are_refused() {
        for source in [
            "C:/Users/me/Pictures/secret.png",
            "file:///etc/passwd",
            "/data/cat.jpg",
            "https://169.254.169.254/latest/meta-data",
        ] {
            let image = chat_with(&json!({ "type": "image_url", "image_url": { "url": source } }));
            assert!(
                check_body(&image, false).is_err(),
                "{source} should be refused"
            );
        }
    }

    #[test]
    fn grammar_files_and_draft_model_paths_are_refused() {
        assert!(check_body(&json!({ "grammar_path": "C:/x.gbnf" }), false).is_err());
        assert!(
            check_body(
                &json!({ "spec_draft_model": "D:\\models\\draft.gguf" }),
                false
            )
            .is_err()
        );
        assert_eq!(
            check_body(
                &json!({ "spec_draft_model": "unsloth/Qwen3-0.6B-GGUF" }),
                false
            ),
            Ok(())
        );
        assert_eq!(
            check_body(&json!({ "grammar_string": "root ::= \"a\"" }), false),
            Ok(())
        );
    }

    #[test]
    fn trusted_callers_may_send_anything() {
        let image =
            chat_with(&json!({ "type": "image_url", "image_url": { "url": "C:/cat.png" } }));
        assert_eq!(check_body(&image, true), Ok(()));
    }
}
