//! Battery saver: on battery, chat requests make fewer tokens, so the
//! laptop lasts longer. Each token costs about the same energy whatever the
//! power mode (measured on GenieX 0.12: power modes change nothing yet), so
//! what saves energy is not thinking and shorter replies.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

/// The only endpoint the saver changes.
const CHAT_PATH: &str = "/v1/chat/completions";
/// Below this, replies are cut too short to be useful.
pub const MIN_REPLY_TOKENS: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct BatterySaver {
    pub enabled: bool,
    /// Turn thinking off: reasoning is often most of a reply's tokens.
    pub no_thinking: bool,
    /// Longest reply, in tokens (`None`: as asked).
    pub max_reply_tokens: Option<u32>,
    /// For requests that don't choose one. GenieX ignores power modes for
    /// now; this is ready for when it doesn't.
    pub power_mode: String,
}

impl Default for BatterySaver {
    fn default() -> Self {
        Self {
            enabled: false,
            no_thinking: true,
            max_reply_tokens: Some(512),
            power_mode: "power_saver".into(),
        }
    }
}

impl BatterySaver {
    /// Change `request` (a chat request) as the saver says; whether it
    /// changed. Thinking and length apply to every chat request, the power
    /// mode only to those that choose none.
    pub(crate) fn apply(&self, upstream_path: &str, request: &mut Map<String, Value>) -> bool {
        if !self.enabled || upstream_path != CHAT_PATH {
            return false;
        }
        let before = request.clone();
        if self.no_thinking {
            request.insert("enable_think".into(), Value::Bool(false));
        }
        if let Some(cap) = self.max_reply_tokens {
            for field in ["max_tokens", "max_completion_tokens"] {
                if let Some(asked) = request.get(field).and_then(Value::as_u64) {
                    request.insert(field.into(), Value::from(asked.min(u64::from(cap))));
                }
            }
            if !request.contains_key("max_tokens") && !request.contains_key("max_completion_tokens")
            {
                request.insert("max_tokens".into(), Value::from(cap));
            }
        }
        if !request.contains_key("power_mode") {
            request.insert("power_mode".into(), Value::String(self.power_mode.clone()));
        }
        *request != before
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn on() -> BatterySaver {
        BatterySaver {
            enabled: true,
            ..BatterySaver::default()
        }
    }

    fn applied(saver: &BatterySaver, path: &str, request: &Value) -> Value {
        let mut request = request.as_object().unwrap().clone();
        saver.apply(path, &mut request);
        Value::Object(request)
    }

    #[test]
    fn makes_fewer_tokens() {
        let request = json!({ "model": "m", "enable_think": true, "max_tokens": 4096 });
        assert_eq!(
            applied(&on(), CHAT_PATH, &request),
            json!({ "model": "m", "enable_think": false, "max_tokens": 512, "power_mode": "power_saver" })
        );
        // A shorter limit stays; none gets the cap.
        let short = applied(&on(), CHAT_PATH, &json!({ "max_completion_tokens": 100 }));
        assert_eq!(short["max_completion_tokens"], 100);
        assert!(short.get("max_tokens").is_none());
        assert_eq!(applied(&on(), CHAT_PATH, &json!({}))["max_tokens"], 512);
    }

    #[test]
    fn keeps_what_it_should() {
        let chosen = applied(&on(), CHAT_PATH, &json!({ "power_mode": "burst" }));
        assert_eq!(chosen["power_mode"], "burst");

        let thinking = BatterySaver {
            no_thinking: false,
            max_reply_tokens: None,
            ..on()
        };
        let request = applied(&thinking, CHAT_PATH, &json!({ "enable_think": true }));
        assert_eq!(
            request,
            json!({ "enable_think": true, "power_mode": "power_saver" })
        );

        let mut request = json!({ "model": "m" }).as_object().unwrap().clone();
        assert!(!on().apply("/v1/completions", &mut request));
        assert!(!BatterySaver::default().apply(CHAT_PATH, &mut request));
    }
}
