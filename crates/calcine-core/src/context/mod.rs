//! How much of a model's context window a conversation takes, and which
//! messages to forget when it doesn't fit.
//!
//! AI Hub models have their window compiled in (4096 tokens for most), and
//! GenieX refuses a prompt that doesn't fit. Counting the prompt first lets
//! Calcine show how full a conversation is and, when asked, forget the
//! oldest messages instead of failing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

use crate::Result;

/// Tokens a chat request takes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TokenCount {
    /// Each message, in order, with its share of the chat template.
    pub per_message: Vec<u32>,
    /// The rest of the prompt: tools, the start of the reply, the default
    /// system prompt GenieX adds when there's none.
    pub overhead: u32,
    /// Counted with the model's tokenizer (within a few tokens of what
    /// GenieX counts) rather than estimated from the text's length.
    pub exact: bool,
}

impl TokenCount {
    pub fn total(&self) -> u32 {
        self.per_message.iter().sum::<u32>() + self.overhead
    }
}

/// How full a conversation is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContextUsage {
    pub tokens: TokenCount,
    /// The model's context window, when known.
    pub window: Option<u32>,
}

/// Counts tokens with each model's own tokenizer.
#[async_trait]
pub trait ContextMeter: Send + Sync {
    /// The window compiled into an AI Hub model. `None` for llama.cpp models,
    /// whose window is a server option.
    async fn compiled_window(&self, model: &str) -> Result<Option<u32>>;

    /// Tokens a chat completion request (`messages`, `tools`, `enable_think`)
    /// takes.
    async fn count(&self, model: &str, request: &Value) -> Result<TokenCount>;
}

/// Room left for the reply: what the request asks for, at most half the
/// window, 512 tokens when it doesn't say.
pub fn reply_reserve(request: &Value, window: u32) -> u32 {
    let asked = ["max_completion_tokens", "max_tokens"]
        .iter()
        .find_map(|key| request.get(*key).and_then(Value::as_u64))
        .map_or(512, |tokens| u32::try_from(tokens).unwrap_or(u32::MAX));
    asked.min(window / 2)
}

/// Where the kept conversation starts so the prompt fits `budget` tokens:
/// leading system messages stay, the oldest exchanges go first, and the kept
/// part starts at a user message. `Some(0)` when everything fits, `None`
/// when even the last exchange doesn't.
pub fn first_kept(messages: &[Value], count: &TokenCount, budget: u32) -> Option<usize> {
    if count.total() <= budget {
        return Some(0);
    }
    let role = |index: usize| messages[index].get("role").and_then(Value::as_str);
    let leading = (0..messages.len())
        .take_while(|&index| matches!(role(index), Some("system" | "developer")))
        .count();
    let tokens = |index: usize| count.per_message.get(index).copied().unwrap_or(0);
    let fixed = count.overhead + (0..leading).map(tokens).sum::<u32>();
    let mut kept: u32 = (leading..messages.len()).map(tokens).sum();
    for start in leading..messages.len() {
        if start > leading {
            kept -= tokens(start - 1);
        }
        if role(start) == Some("user") && fixed + kept <= budget {
            return Some(start);
        }
    }
    None
}

/// Remove what [`first_kept`] leaves out: how many messages were forgotten,
/// or `None` when even the last exchange doesn't fit.
pub fn forget_oldest(messages: &mut Vec<Value>, count: &TokenCount, budget: u32) -> Option<usize> {
    let start = first_kept(messages, count, budget)?;
    let leading = messages
        .iter()
        .take_while(|message| {
            matches!(
                message.get("role").and_then(Value::as_str),
                Some("system" | "developer")
            )
        })
        .count();
    if start <= leading {
        return Some(0);
    }
    messages.drain(leading..start);
    Some(start - leading)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn conversation() -> (Vec<Value>, TokenCount) {
        let messages = vec![
            json!({ "role": "system", "content": "Be brief." }),
            json!({ "role": "user", "content": "one" }),
            json!({ "role": "assistant", "content": "", "tool_calls": [] }),
            json!({ "role": "tool", "content": "result" }),
            json!({ "role": "assistant", "content": "two" }),
            json!({ "role": "user", "content": "three" }),
            json!({ "role": "assistant", "content": "four" }),
            json!({ "role": "user", "content": "five" }),
        ];
        let count = TokenCount {
            per_message: vec![10, 100, 20, 50, 30, 100, 100, 100],
            overhead: 20,
            exact: true,
        };
        (messages, count)
    }

    #[test]
    fn keeps_everything_that_fits() {
        let (messages, count) = conversation();
        assert_eq!(count.total(), 530);
        assert_eq!(first_kept(&messages, &count, 530), Some(0));
    }

    #[test]
    fn forgets_whole_exchanges_from_the_oldest() {
        let (messages, count) = conversation();
        // Without the first exchange (and its tool call): 30 + 300.
        assert_eq!(first_kept(&messages, &count, 529), Some(5));
        assert_eq!(first_kept(&messages, &count, 330), Some(5));
        // Only the last question.
        assert_eq!(first_kept(&messages, &count, 329), Some(7));
        assert_eq!(first_kept(&messages, &count, 130), Some(7));
        assert_eq!(first_kept(&messages, &count, 129), None);
    }

    #[test]
    fn removes_the_forgotten_messages() {
        let (mut messages, count) = conversation();
        assert_eq!(forget_oldest(&mut messages, &count, 330), Some(4));
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["content"], "three");

        let (mut messages, count) = conversation();
        assert_eq!(forget_oldest(&mut messages, &count, 1000), Some(0));
        assert_eq!(messages.len(), 8);
        assert_eq!(forget_oldest(&mut messages, &count, 10), None);
    }

    #[test]
    fn reserves_room_for_the_reply() {
        assert_eq!(reply_reserve(&json!({}), 4096), 512);
        assert_eq!(reply_reserve(&json!({ "max_tokens": 100 }), 4096), 100);
        assert_eq!(
            reply_reserve(&json!({ "max_completion_tokens": 9000 }), 4096),
            2048
        );
    }
}
