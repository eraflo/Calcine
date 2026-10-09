//! Token counts against a real cached model (skipped without one).

use calcine_core::context::ContextMeter;
use calcine_geniex::{Geniex, GeniexContext};
use serde_json::json;

#[tokio::test]
#[ignore = "needs qualcomm/Qwen3-0.6B in the GenieX cache"]
async fn matches_what_geniex_reports() {
    let context = GeniexContext::new(Geniex::default());
    let model = "qualcomm/Qwen3-0.6B";
    assert_eq!(context.compiled_window(model).await.unwrap(), Some(4096));
    // Prompt sizes `geniex serve` reported for these requests.
    for (request, reported) in [
        (
            json!({ "enable_think": false, "messages": [{ "role": "user", "content": "Hello" }] }),
            26,
        ),
        (
            json!({ "enable_think": false, "messages": [{ "role": "user", "content": "Hello world how are you" }] }),
            30,
        ),
        (
            json!({ "enable_think": false, "messages": [{ "role": "system", "content": "Be brief." }, { "role": "user", "content": "Hello" }] }),
            22,
        ),
        (
            json!({ "enable_think": false, "messages": [{ "role": "user", "content": "Hello" }, { "role": "assistant", "content": "Hi there" }, { "role": "user", "content": "Bye" }] }),
            40,
        ),
        (
            json!({ "enable_think": true, "messages": [{ "role": "user", "content": "Hello" }] }),
            22,
        ),
    ] {
        let count = context.count(model, &request).await.unwrap();
        assert!(count.exact);
        let total = i64::from(count.total());
        assert!(
            (total - reported).abs() <= 2,
            "{request}: {total} vs {reported}"
        );
    }
}
