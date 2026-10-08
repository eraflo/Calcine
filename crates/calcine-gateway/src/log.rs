//! Recent requests, for the Server page. Prompts are never recorded.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use serde_json::Value;
use specta::Type;
use tokio::sync::broadcast;

use crate::now_ms;

const KEPT_ENTRIES: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestEntry {
    pub id: u32,
    pub started_at_ms: u64,
    /// The API key's name, `Calcine`, or `No key`.
    pub client: String,
    pub method: String,
    pub path: String,
    pub model: Option<String>,
    pub stream: bool,
    /// `None` while in flight.
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    /// Time until the first streamed bytes.
    pub first_token_ms: Option<u64>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub tokens_per_second: Option<f64>,
    pub error: Option<String>,
}

impl RequestEntry {
    pub fn new(
        client: String,
        method: &str,
        path: &str,
        model: Option<String>,
        stream: bool,
    ) -> Self {
        Self {
            id: 0,
            started_at_ms: now_ms(),
            client,
            method: method.to_owned(),
            path: path.to_owned(),
            model,
            stream,
            status: None,
            duration_ms: None,
            first_token_ms: None,
            prompt_tokens: None,
            completion_tokens: None,
            tokens_per_second: None,
            error: None,
        }
    }
}

#[derive(Debug)]
pub struct RequestLog {
    entries: Mutex<VecDeque<RequestEntry>>,
    next_id: AtomicU32,
    events: broadcast::Sender<RequestEntry>,
}

impl Default for RequestLog {
    fn default() -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(KEPT_ENTRIES)),
            next_id: AtomicU32::new(1),
            events: broadcast::channel(128).0,
        }
    }
}

impl RequestLog {
    pub fn start(&self, mut entry: RequestEntry) -> u32 {
        entry.id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = entry.id;
        {
            let mut entries = self.lock();
            if entries.len() == KEPT_ENTRIES {
                entries.pop_back();
            }
            entries.push_front(entry.clone());
        }
        let _ = self.events.send(entry);
        id
    }

    pub fn update(&self, id: u32, change: impl FnOnce(&mut RequestEntry)) {
        let updated = {
            let mut entries = self.lock();
            let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) else {
                return;
            };
            change(entry);
            entry.clone()
        };
        let _ = self.events.send(updated);
    }

    /// Newest first.
    pub fn list(&self) -> Vec<RequestEntry> {
        self.lock().iter().cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RequestEntry> {
        self.events.subscribe()
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<RequestEntry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Token counts and speed reported by GenieX (`usage`, `timings`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Usage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub tokens_per_second: Option<f64>,
}

impl Usage {
    pub fn from_json(value: &Value) -> Self {
        Self {
            prompt_tokens: value
                .pointer("/usage/prompt_tokens")
                .and_then(Value::as_u64),
            completion_tokens: value
                .pointer("/usage/completion_tokens")
                .and_then(Value::as_u64),
            tokens_per_second: value
                .pointer("/timings/predicted_per_second")
                .and_then(Value::as_f64),
        }
    }

    fn merge(&mut self, other: Self) {
        self.prompt_tokens = other.prompt_tokens.or(self.prompt_tokens);
        self.completion_tokens = other.completion_tokens.or(self.completion_tokens);
        self.tokens_per_second = other.tokens_per_second.or(self.tokens_per_second);
    }

    pub fn apply(self, entry: &mut RequestEntry) {
        entry.prompt_tokens = self.prompt_tokens;
        entry.completion_tokens = self.completion_tokens;
        entry.tokens_per_second = self.tokens_per_second;
    }
}

/// Picks `usage`/`timings` out of an SSE stream as it passes through.
#[derive(Debug, Default)]
pub struct SseUsage {
    pending: Vec<u8>,
    pub usage: Usage,
}

impl SseUsage {
    pub fn feed(&mut self, chunk: &[u8]) {
        self.pending.extend_from_slice(chunk);
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let Ok(line) = std::str::from_utf8(&line) else {
                continue;
            };
            let Some(data) = line.trim().strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data.starts_with('{')
                && (data.contains("\"usage\"") || data.contains("\"timings\""))
                && let Ok(value) = serde_json::from_str::<Value>(data)
            {
                self.usage.merge(Usage::from_json(&value));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_usage_from_a_genie_stream() {
        let mut scanner = SseUsage::default();
        let stream = concat!(
            "data:{\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
            "data:{\"choices\":[],\"usage\":{\"completion_tokens\":2,\"prompt_tokens\":30},",
            "\"timings\":{\"predicted_per_second\":143.1}}\n\n",
            "data:[DONE]\n\n",
        );
        // Split mid-line to mimic network chunks.
        let (a, b) = stream.split_at(70);
        scanner.feed(a.as_bytes());
        scanner.feed(b.as_bytes());
        assert_eq!(scanner.usage.prompt_tokens, Some(30));
        assert_eq!(scanner.usage.completion_tokens, Some(2));
        assert_eq!(scanner.usage.tokens_per_second, Some(143.1));
    }

    #[test]
    fn keeps_the_newest_entries_first() {
        let log = RequestLog::default();
        let first = log.start(RequestEntry::new(
            "A".into(),
            "POST",
            "/v1/chat/completions",
            None,
            true,
        ));
        let second = log.start(RequestEntry::new(
            "B".into(),
            "GET",
            "/v1/models",
            None,
            false,
        ));
        log.update(first, |entry| entry.status = Some(200));
        let entries = log.list();
        assert_eq!(entries[0].id, second);
        assert_eq!(entries[1].status, Some(200));
    }
}
