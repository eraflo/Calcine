//! Battery saver: on battery, chat requests that don't choose a power mode
//! run in the one the user picked, so the laptop lasts longer.

use axum::body::Bytes;
use calcine_core::hardware::PowerSource;
use serde_json::Value;

/// The only endpoint GenieX reads `power_mode` on.
const CHAT_PATH: &str = "/v1/chat/completions";

/// The body to send instead of the caller's, when the battery saver applies:
/// `mode` is set, the laptop runs on battery, and the request chose no mode.
pub(crate) fn saver_body(
    upstream_path: &str,
    json: &Value,
    mode: Option<&str>,
    source: Option<PowerSource>,
) -> Option<Bytes> {
    let mode = mode?;
    if upstream_path != CHAT_PATH
        || !source.is_some_and(|source| source.on_battery)
        || json.get("power_mode").is_some()
    {
        return None;
    }
    let mut json = json.as_object()?.clone();
    json.insert("power_mode".into(), Value::from(mode));
    serde_json::to_vec(&json).ok().map(Bytes::from)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const ON_BATTERY: Option<PowerSource> = Some(PowerSource {
        on_battery: true,
        battery_percent: Some(40),
    });
    const ON_MAINS: Option<PowerSource> = Some(PowerSource {
        on_battery: false,
        battery_percent: Some(40),
    });

    fn sent(body: Option<Bytes>) -> Option<Value> {
        body.map(|bytes| serde_json::from_slice(&bytes).unwrap())
    }

    #[test]
    fn sets_the_mode_on_battery() {
        let request = json!({ "model": "m", "messages": [] });
        let body = saver_body(CHAT_PATH, &request, Some("power_saver"), ON_BATTERY);
        assert_eq!(
            sent(body),
            Some(json!({ "model": "m", "messages": [], "power_mode": "power_saver" }))
        );
    }

    #[test]
    fn leaves_the_request_alone_otherwise() {
        let request = json!({ "model": "m", "messages": [] });
        let mode = Some("power_saver");
        assert!(saver_body(CHAT_PATH, &request, mode, ON_MAINS).is_none());
        assert!(saver_body(CHAT_PATH, &request, mode, None).is_none());
        assert!(saver_body(CHAT_PATH, &request, None, ON_BATTERY).is_none());
        assert!(saver_body("/v1/completions", &request, mode, ON_BATTERY).is_none());
        let chosen = json!({ "model": "m", "power_mode": "burst" });
        assert!(saver_body(CHAT_PATH, &chosen, mode, ON_BATTERY).is_none());
    }
}
