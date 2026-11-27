//! Battery saver: on battery, chat requests that don't choose a power mode
//! run in the one the user picked, so the laptop lasts longer.

use serde_json::Value;

/// The only endpoint GenieX reads `power_mode` on.
const CHAT_PATH: &str = "/v1/chat/completions";

/// Whether the saver may set the request's power mode: a chat request
/// that chose none.
pub(crate) fn may_set_mode(upstream_path: &str, json: &Value) -> bool {
    upstream_path == CHAT_PATH && json.is_object() && json.get("power_mode").is_none()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn only_chat_requests_without_a_mode() {
        let request = json!({ "model": "m", "messages": [] });
        assert!(may_set_mode(CHAT_PATH, &request));
        assert!(!may_set_mode("/v1/completions", &request));
        let chosen = json!({ "model": "m", "power_mode": "burst" });
        assert!(!may_set_mode(CHAT_PATH, &chosen));
        assert!(!may_set_mode(CHAT_PATH, &json!([])));
    }
}
