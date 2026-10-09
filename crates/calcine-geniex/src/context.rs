//! Context windows and token counts for cached models.
//!
//! AI Hub models carry their window in `genie_config.json` and their
//! tokenizer in `tokenizer.json`. The chat template's share is a few
//! constants, measured against the prompt sizes `geniex serve` reports for
//! Qwen3 (ChatML), within a couple of tokens. llama.cpp models keep their
//! tokenizer inside the GGUF file, so their counts are estimated.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use calcine_core::context::{ContextMeter, TokenCount};
use calcine_core::runtime::RuntimeManager;
use calcine_core::{Error, Result};
use serde_json::Value;
use tokenizers::Tokenizer;

use crate::Geniex;

/// `<|im_start|>` and `<|im_end|>\n` around each message (its role and
/// newline are counted with its text).
const PER_MESSAGE: u32 = 3;
/// `<|im_start|>assistant\n`, where the reply starts.
const REPLY_START: u32 = 3;
/// The empty `<think></think>` block when thinking is off.
const EMPTY_THINK: u32 = 4;
/// The system prompt GenieX adds when the request has none.
const DEFAULT_SYSTEM: u32 = 13;
/// The template's explanation of how to call tools.
const TOOLS_PREAMBLE: u32 = 90;
/// What an image costs, roughly, for vision models.
const IMAGE: u32 = 256;

/// Counts tokens with each cached model's tokenizer.
#[derive(Debug)]
pub struct GeniexContext {
    geniex: Geniex,
    tokenizers: Mutex<HashMap<PathBuf, Arc<Tokenizer>>>,
}

impl GeniexContext {
    pub fn new(geniex: Geniex) -> Self {
        Self {
            geniex,
            tokenizers: Mutex::new(HashMap::new()),
        }
    }

    /// The model's folder in the cache, for names like `qualcomm/Qwen3-4B`
    /// or `qualcomm/Qwen3-4B:W4A16`. Names come from API clients: anything
    /// that could leave the cache is refused.
    fn model_dir(&self, model: &str) -> Result<Option<PathBuf>> {
        let name = model.split(':').next().unwrap_or(model);
        let safe = |part: &str| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        };
        if !name.split('/').all(safe) {
            return Err(Error::InvalidInput(format!("{model} isn't a model name")));
        }
        Ok(self
            .geniex
            .models_dir()
            .map(|dir| name.split('/').fold(dir, |path, part| path.join(part)))
            .filter(|dir| dir.is_dir()))
    }

    async fn tokenizer(&self, dir: &Path) -> Option<Arc<Tokenizer>> {
        let path = dir.join("tokenizer.json");
        if let Some(loaded) = self
            .tokenizers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&path)
        {
            return Some(loaded.clone());
        }
        if !path.is_file() {
            return None;
        }
        let file = path.clone();
        let loaded = tokio::task::spawn_blocking(move || Tokenizer::from_file(file))
            .await
            .ok()?
            .inspect_err(|err| tracing::warn!(%err, path = %path.display(), "can't load tokenizer"))
            .ok()
            .map(Arc::new)?;
        self.tokenizers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(path, loaded.clone());
        Some(loaded)
    }
}

#[async_trait]
impl ContextMeter for GeniexContext {
    async fn compiled_window(&self, model: &str) -> Result<Option<u32>> {
        let Some(dir) = self.model_dir(model)? else {
            return Ok(None);
        };
        let Ok(text) = tokio::fs::read_to_string(dir.join("genie_config.json")).await else {
            return Ok(None);
        };
        let config: Value = serde_json::from_str(&text)
            .map_err(|err| Error::Parse(format!("genie_config.json: {err}")))?;
        Ok(context_size(&config))
    }

    async fn count(&self, model: &str, request: &Value) -> Result<TokenCount> {
        let tokenizer = match self.model_dir(model)? {
            Some(dir) => self.tokenizer(&dir).await,
            None => None,
        };
        let parts = Parts::of(request);
        let texts = parts.texts();
        let counted: Option<Vec<u32>> = match tokenizer {
            Some(tokenizer) => tokio::task::spawn_blocking(move || {
                tokenizer.encode_batch(texts, false).ok().map(|encodings| {
                    encodings
                        .iter()
                        .map(|encoding| len(encoding.len()))
                        .collect()
                })
            })
            .await
            .map_err(|err| Error::Io(std::io::Error::other(err)))?,
            None => None,
        };
        Ok(if let Some(lengths) = counted {
            parts.count(&lengths, true)
        } else {
            let lengths: Vec<u32> = parts.texts().iter().map(|text| estimate(text)).collect();
            parts.count(&lengths, false)
        })
    }
}

fn len(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// About three characters per token: on the safe side for English and code.
pub fn estimate(text: &str) -> u32 {
    len(text.chars().count().div_ceil(3))
}

/// `dialog.context.size`, wherever the config keeps it.
fn context_size(config: &Value) -> Option<u32> {
    match config {
        Value::Object(fields) => fields
            .get("context")
            .and_then(|context| context.get("size"))
            .and_then(Value::as_u64)
            .and_then(|size| u32::try_from(size).ok())
            .or_else(|| fields.values().find_map(context_size)),
        _ => None,
    }
}

/// The texts of a request, to count in one batch.
#[derive(Debug, Default)]
pub struct Parts {
    /// One text per message.
    messages: Vec<String>,
    /// Images in each message.
    images: Vec<u32>,
    tools: Option<String>,
    has_system: bool,
    thinks: bool,
}

impl Parts {
    pub fn of(request: &Value) -> Self {
        let messages = request
            .get("messages")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut parts = Self {
            tools: request
                .get("tools")
                .and_then(Value::as_array)
                .filter(|tools| !tools.is_empty())
                .map(|tools| Value::Array(tools.clone()).to_string()),
            has_system: messages
                .first()
                .and_then(|message| message.get("role"))
                .and_then(Value::as_str)
                .is_some_and(|role| matches!(role, "system" | "developer")),
            thinks: request
                .get("enable_think")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            ..Self::default()
        };
        for message in messages {
            let (text, images) = message_text(message);
            parts.messages.push(text);
            parts.images.push(images);
        }
        parts
    }

    /// Messages first, then the tools.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = self.messages.clone();
        texts.extend(self.tools.clone());
        texts
    }

    /// The count, given the length of each of `texts()`.
    pub fn count(&self, lengths: &[u32], exact: bool) -> TokenCount {
        let at = |index: usize| lengths.get(index).copied().unwrap_or(0);
        let per_message = (0..self.messages.len())
            .map(|index| at(index) + PER_MESSAGE + self.images[index] * IMAGE)
            .collect();
        let tools = self
            .tools
            .as_ref()
            .map_or(0, |_| TOOLS_PREAMBLE + at(self.messages.len()));
        let overhead = REPLY_START
            + if self.thinks { 0 } else { EMPTY_THINK }
            + if self.has_system { 0 } else { DEFAULT_SYSTEM }
            + tools;
        TokenCount {
            per_message,
            overhead,
            exact: exact && self.images.iter().all(|&images| images == 0),
        }
    }
}

/// A message's text (role, content, tool calls) and how many images it has.
fn message_text(message: &Value) -> (String, u32) {
    let mut text = message
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    text.push('\n');
    let mut images = 0;
    match message.get("content") {
        Some(Value::String(content)) => text.push_str(content),
        Some(Value::Array(content)) => {
            for part in content {
                match part.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        text.push_str(part.get("text").and_then(Value::as_str).unwrap_or(""));
                    }
                    Some("image_url" | "input_audio") => images += 1,
                    _ => {}
                }
            }
        }
        _ => {}
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            text.push_str(
                &call
                    .get("function")
                    .map(Value::to_string)
                    .unwrap_or_default(),
            );
        }
    }
    (text, images)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn finds_the_compiled_window() {
        let config =
            json!({ "dialog": { "version": 1, "context": { "size": 4096, "n-vocab": 151_936 } } });
        assert_eq!(context_size(&config), Some(4096));
        assert_eq!(context_size(&json!({ "dialog": {} })), None);
    }

    #[test]
    fn adds_the_template_around_messages() {
        let request = json!({
            "enable_think": false,
            "messages": [{ "role": "user", "content": "Hello" }],
        });
        let parts = Parts::of(&request);
        // "user\nHello": 3 tokens with Qwen3's tokenizer. GenieX reports 26.
        let count = parts.count(&[3], true);
        assert_eq!(count.per_message, vec![6]);
        assert_eq!(count.total(), 26);
    }

    #[test]
    fn images_make_it_an_estimate() {
        let request = json!({ "messages": [
            { "role": "system", "content": "Be brief." },
            { "role": "user", "content": [
                { "type": "image_url", "image_url": { "url": "data:," } },
                { "type": "text", "text": "What is it?" },
            ] },
        ] });
        let parts = Parts::of(&request);
        assert_eq!(parts.texts()[1], "user\nWhat is it?");
        let count = parts.count(&[4, 5], true);
        assert_eq!(count.per_message[1], 5 + PER_MESSAGE + IMAGE);
        assert_eq!(count.overhead, REPLY_START);
        assert!(!count.exact);
    }

    #[test]
    fn refuses_names_that_leave_the_cache() {
        let context = GeniexContext::new(Geniex::default());
        assert!(context.model_dir("../secrets").is_err());
        assert!(context.model_dir("qualcomm/..").is_err());
        assert!(context.model_dir("..\\Windows").is_err());
        assert!(context.model_dir("qualcomm/Qwen3-4B:W4A16").is_ok());
    }
}
