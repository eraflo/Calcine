//! Ollama's API (`/api/*`) translated to and from GenieX's OpenAI API, for
//! apps that only speak Ollama.
//!
//! Requests become streamed `/v1/chat/completions` calls; the OpenAI
//! server-sent events are turned back into Ollama's JSON lines, or into one
//! JSON object when the client asked for `"stream": false`.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

/// Ollama's default port, where Ollama clients look first.
pub const OLLAMA_PORT: u16 = 11434;

/// Version reported by `/api/version`. Some clients refuse old servers.
pub const COMPAT_VERSION: &str = "0.12.0";

/// `/api/chat` answers with `message`, `/api/generate` with `response`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Chat,
    Generate,
}

/// An Ollama request, ready for GenieX.
#[derive(Debug, Clone, PartialEq)]
pub struct Translated {
    /// OpenAI chat completion body, always streamed.
    pub body: Value,
    pub model: String,
    /// Whether the client wants JSON lines (Ollama streams by default).
    pub stream: bool,
}

/// Ollama names a model `name:tag`; `:latest` is what clients add when a
/// name has no tag, and GenieX ids don't have it.
pub fn model_id(name: &str) -> String {
    name.strip_suffix(":latest").unwrap_or(name).to_owned()
}

/// `/api/chat` → `/v1/chat/completions`.
pub fn chat_request(body: &Value) -> Result<Translated, String> {
    let model = required_model(body)?;
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or("messages is required")?
        .iter()
        .map(message)
        .collect::<Result<Vec<_>, _>>()?;
    let mut request = base_request(&model, body);
    request.insert("messages".into(), Value::Array(messages));
    if let Some(tools) = body.get("tools").filter(|tools| tools.is_array()) {
        request.insert("tools".into(), tools.clone());
    }
    Ok(Translated {
        body: Value::Object(request),
        model,
        stream: wants_stream(body),
    })
}

/// `/api/generate` → `/v1/chat/completions`, with `system` and `prompt`
/// as messages. An empty prompt only loads the model (see [`is_load_only`]).
pub fn generate_request(body: &Value) -> Result<Translated, String> {
    let model = required_model(body)?;
    let mut messages = Vec::new();
    if let Some(system) = body.get("system").and_then(Value::as_str)
        && !system.is_empty()
    {
        messages.push(json!({ "role": "system", "content": system }));
    }
    let user = json!({
        "role": "user",
        "content": body.get("prompt").and_then(Value::as_str).unwrap_or(""),
        "images": body.get("images").cloned().unwrap_or(Value::Null),
    });
    messages.push(message(&user)?);
    let mut request = base_request(&model, body);
    request.insert("messages".into(), Value::Array(messages));
    Ok(Translated {
        body: Value::Object(request),
        model,
        stream: wants_stream(body),
    })
}

/// Ollama loads a model when `/api/generate` gets an empty prompt.
pub fn is_load_only(body: &Value) -> bool {
    body.get("prompt")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
        && body
            .get("images")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
}

fn required_model(body: &Value) -> Result<String, String> {
    body.get("model")
        .and_then(Value::as_str)
        .filter(|model| !model.is_empty())
        .map(model_id)
        .ok_or_else(|| "model is required".to_owned())
}

fn wants_stream(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(true)
}

/// Fields shared by chat and generate: model, options, thinking, format.
fn base_request(model: &str, body: &Value) -> Map<String, Value> {
    let mut request = Map::new();
    request.insert("model".into(), json!(model));
    request.insert("stream".into(), json!(true));
    request.insert("stream_options".into(), json!({ "include_usage": true }));
    // Keep the chain of thought out of the answer, in `reasoning_content`.
    request.insert("reasoning_format".into(), json!("deepseek"));
    if let Some(think) = body.get("think") {
        // `true`, `false`, or a level (`"high"`, …) that GenieX doesn't have.
        request.insert("enable_think".into(), json!(think.as_bool() != Some(false)));
    }
    match body.get("format") {
        Some(Value::String(format)) if format == "json" => {
            request.insert("response_format".into(), json!({ "type": "json_object" }));
        }
        Some(schema @ Value::Object(_)) => {
            request.insert(
                "response_format".into(),
                json!({ "type": "json_schema", "json_schema": { "name": "response", "schema": schema } }),
            );
        }
        _ => {}
    }
    if let Some(options) = body.get("options").and_then(Value::as_object) {
        for (from, to) in OPTIONS {
            if let Some(value) = options.get(*from).filter(|value| !value.is_null()) {
                request.insert((*to).into(), value.clone());
            }
        }
        // Ollama uses -1 (infinite) and -2 (fill the context) for no limit.
        if let Some(limit) = options.get("num_predict").and_then(Value::as_i64)
            && limit > 0
        {
            request.insert("max_tokens".into(), json!(limit));
        }
    }
    request
}

/// Ollama `options` with an OpenAI or GenieX equivalent. `num_ctx` isn't
/// here: the context window is a server option in Calcine.
const OPTIONS: &[(&str, &str)] = &[
    ("temperature", "temperature"),
    ("top_p", "top_p"),
    ("top_k", "top_k"),
    ("min_p", "min_p"),
    ("seed", "seed"),
    ("stop", "stop"),
    ("repeat_penalty", "repetition_penalty"),
    ("presence_penalty", "presence_penalty"),
    ("frequency_penalty", "frequency_penalty"),
];

/// One Ollama message as an OpenAI message: images become `image_url`
/// parts, tool calls get the OpenAI shape.
fn message(message: &Value) -> Result<Value, String> {
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .ok_or("every message needs a role")?;
    let text = message.get("content").and_then(Value::as_str).unwrap_or("");
    let images: Vec<&str> = message
        .get("images")
        .and_then(Value::as_array)
        .map(|images| images.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let content = if images.is_empty() {
        json!(text)
    } else {
        let mut parts: Vec<Value> = images
            .iter()
            .map(|image| json!({ "type": "image_url", "image_url": { "url": data_url(image) } }))
            .collect();
        if !text.is_empty() {
            parts.push(json!({ "type": "text", "text": text }));
        }
        Value::Array(parts)
    };
    let mut out = Map::new();
    out.insert("role".into(), json!(role));
    out.insert("content".into(), content);
    if let Some(name) = message.get("tool_name").and_then(Value::as_str) {
        out.insert("name".into(), json!(name));
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        let calls: Vec<Value> = calls
            .iter()
            .enumerate()
            .map(|(index, call)| {
                let function = call.get("function").cloned().unwrap_or(Value::Null);
                let arguments = match function.get("arguments") {
                    Some(Value::String(text)) => text.clone(),
                    Some(value) => value.to_string(),
                    None => "{}".to_owned(),
                };
                json!({
                    "id": format!("call_{index}"),
                    "type": "function",
                    "function": { "name": function.get("name").cloned().unwrap_or(Value::Null), "arguments": arguments },
                })
            })
            .collect();
        out.insert("tool_calls".into(), Value::Array(calls));
    }
    Ok(Value::Object(out))
}

/// Ollama sends images as bare base64; OpenAI wants a data URL. The type
/// is read from the first bytes.
fn data_url(base64: &str) -> String {
    if base64.starts_with("data:") {
        return base64.to_owned();
    }
    let mime = match base64.get(..4) {
        Some("iVBO") => "image/png",
        Some("R0lG") => "image/gif",
        Some("UklG") => "image/webp",
        _ => "image/jpeg",
    };
    format!("data:{mime};base64,{base64}")
}

/// Builds Ollama's answer from GenieX's server-sent events.
#[derive(Debug)]
pub struct Answer {
    model: String,
    mode: Mode,
    pending: String,
    content: String,
    thinking: String,
    tool_calls: Vec<ToolCall>,
    done_reason: Option<String>,
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    prompt_ms: Option<f64>,
    predicted_ms: Option<f64>,
    tokens_per_second: Option<f64>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCall {
    name: String,
    arguments: String,
}

impl Answer {
    pub fn new(model: &str, mode: Mode) -> Self {
        Self {
            model: model.to_owned(),
            mode,
            pending: String::new(),
            content: String::new(),
            thinking: String::new(),
            tool_calls: Vec::new(),
            done_reason: None,
            prompt_tokens: None,
            completion_tokens: None,
            prompt_ms: None,
            predicted_ms: None,
            tokens_per_second: None,
            error: None,
        }
    }

    /// Read more of the event stream; returns the text pieces to stream to
    /// the client, as Ollama chunks.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Value> {
        self.pending.push_str(&String::from_utf8_lossy(bytes));
        let mut chunks = Vec::new();
        while let Some(newline) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=newline).collect();
            let Some(data) = line.trim_end().strip_prefix("data:") else {
                continue;
            };
            if let Some(chunk) = self.event(data.trim()) {
                chunks.push(chunk);
            }
        }
        chunks
    }

    fn event(&mut self, data: &str) -> Option<Value> {
        let event: Value = serde_json::from_str(data).ok()?;
        if let Some(error) = event.get("error") {
            self.error = Some(match error {
                Value::String(message) => message.clone(),
                other => other
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("generation failed")
                    .to_owned(),
            });
            return None;
        }
        if let Some(usage) = event.get("usage") {
            self.prompt_tokens = usage.get("prompt_tokens").and_then(Value::as_u64);
            self.completion_tokens = usage.get("completion_tokens").and_then(Value::as_u64);
        }
        if let Some(timings) = event.get("timings") {
            self.prompt_ms = timings.get("prompt_ms").and_then(Value::as_f64);
            self.predicted_ms = timings.get("predicted_ms").and_then(Value::as_f64);
            self.tokens_per_second = timings.get("predicted_per_second").and_then(Value::as_f64);
        }
        let choice = event.pointer("/choices/0")?;
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.done_reason = Some(
                if reason == "tool_calls" {
                    "stop"
                } else {
                    reason
                }
                .to_owned(),
            );
        }
        let delta = choice.get("delta")?;
        for call in delta
            .get("tool_calls")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let index = call
                .get("index")
                .and_then(Value::as_u64)
                .and_then(|index| usize::try_from(index).ok())
                .unwrap_or(self.tool_calls.len());
            if self.tool_calls.len() <= index {
                self.tool_calls.resize_with(index + 1, ToolCall::default);
            }
            let slot = &mut self.tool_calls[index];
            if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                slot.name.push_str(name);
            }
            if let Some(arguments) = call.pointer("/function/arguments").and_then(Value::as_str) {
                slot.arguments.push_str(arguments);
            }
        }
        let content = delta.get("content").and_then(Value::as_str).unwrap_or("");
        let thinking = delta
            .get("reasoning_content")
            .and_then(Value::as_str)
            .unwrap_or("");
        if content.is_empty() && thinking.is_empty() {
            return None;
        }
        self.content.push_str(content);
        self.thinking.push_str(thinking);
        Some(self.chunk(content, thinking, false))
    }

    /// Why generation failed, if GenieX said so inside the stream.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn prompt_tokens(&self) -> Option<u64> {
        self.prompt_tokens
    }

    pub fn completion_tokens(&self) -> Option<u64> {
        self.completion_tokens
    }

    pub fn tokens_per_second(&self) -> Option<f64> {
        self.tokens_per_second
    }

    /// The last line of a stream, with statistics.
    pub fn done(&self, total_ms: u64) -> Value {
        self.finish(total_ms, "", "")
    }

    /// The whole answer in one object, for `"stream": false`.
    pub fn whole(&self, total_ms: u64) -> Value {
        self.finish(total_ms, &self.content, &self.thinking)
    }

    fn finish(&self, total_ms: u64, content: &str, thinking: &str) -> Value {
        let mut chunk = self.chunk(content, thinking, true);
        let object = chunk.as_object_mut().expect("chunks are objects");
        if !self.tool_calls.is_empty() && self.mode == Mode::Chat {
            let calls: Vec<Value> = self
                .tool_calls
                .iter()
                .map(|call| {
                    let arguments = serde_json::from_str::<Value>(&call.arguments)
                        .unwrap_or_else(|_| json!({}));
                    json!({ "function": { "name": call.name, "arguments": arguments } })
                })
                .collect();
            object["message"]["tool_calls"] = Value::Array(calls);
        }
        let millis_to_nanos = |ms: f64| {
            std::time::Duration::try_from_secs_f64(ms / 1000.0).map_or(0, |duration| {
                u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
            })
        };
        object.insert(
            "done_reason".into(),
            json!(self.done_reason.as_deref().unwrap_or("stop")),
        );
        object.insert("total_duration".into(), json!(total_ms * 1_000_000));
        object.insert("load_duration".into(), json!(0));
        object.insert(
            "prompt_eval_count".into(),
            json!(self.prompt_tokens.unwrap_or(0)),
        );
        object.insert(
            "prompt_eval_duration".into(),
            json!(self.prompt_ms.map_or(0, millis_to_nanos)),
        );
        object.insert(
            "eval_count".into(),
            json!(self.completion_tokens.unwrap_or(0)),
        );
        object.insert(
            "eval_duration".into(),
            json!(self.predicted_ms.map_or(0, millis_to_nanos)),
        );
        chunk
    }

    fn chunk(&self, content: &str, thinking: &str, done: bool) -> Value {
        let mut chunk = json!({ "model": self.model, "created_at": now_rfc3339() });
        match self.mode {
            Mode::Chat => {
                let mut message = json!({ "role": "assistant", "content": content });
                if !thinking.is_empty() {
                    message["thinking"] = json!(thinking);
                }
                chunk["message"] = message;
            }
            Mode::Generate => {
                chunk["response"] = json!(content);
                if !thinking.is_empty() {
                    chunk["thinking"] = json!(thinking);
                }
            }
        }
        chunk["done"] = json!(done);
        chunk
    }
}

/// The reply to `/api/generate` with an empty prompt, once the model is loaded.
pub fn loaded(model: &str) -> Value {
    json!({
        "model": model,
        "created_at": now_rfc3339(),
        "response": "",
        "done": true,
        "done_reason": "load",
    })
}

/// Current UTC time as RFC 3339, like Ollama's `created_at`.
pub fn now_rfc3339() -> String {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    rfc3339(since_epoch.as_secs(), since_epoch.subsec_nanos())
}

/// `secs` since the Unix epoch as `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`.
fn rfc3339(secs: u64, nanos: u32) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rest = secs % 86_400;
    // Howard Hinnant's days-to-civil algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{nanos:09}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn translates_a_chat_request() {
        let translated = chat_request(&json!({
            "model": "qualcomm/Qwen3-4B:latest",
            "messages": [
                { "role": "system", "content": "Be brief." },
                { "role": "user", "content": "What is this?", "images": ["iVBORw0KGgo="] },
            ],
            "options": { "temperature": 0.2, "num_predict": 64, "repeat_penalty": 1.1, "num_ctx": 8192 },
            "think": false,
            "format": "json",
        }))
        .unwrap();
        assert_eq!(translated.model, "qualcomm/Qwen3-4B");
        assert!(translated.stream);
        let body = translated.body;
        assert_eq!(body["stream"], true);
        assert_eq!(body["temperature"], 0.2);
        assert_eq!(body["max_tokens"], 64);
        assert_eq!(body["repetition_penalty"], 1.1);
        assert!(body.get("num_ctx").is_none());
        assert_eq!(body["enable_think"], false);
        assert_eq!(body["response_format"]["type"], "json_object");
        assert_eq!(
            body["messages"][0],
            json!({ "role": "system", "content": "Be brief." })
        );
        assert_eq!(
            body["messages"][1]["content"][0]["image_url"]["url"],
            "data:image/png;base64,iVBORw0KGgo="
        );
        assert_eq!(body["messages"][1]["content"][1]["text"], "What is this?");
    }

    #[test]
    fn no_limit_and_no_think_leave_geniex_defaults() {
        let body = chat_request(&json!({
            "model": "m",
            "messages": [],
            "options": { "num_predict": -1 },
            "stream": false,
        }))
        .unwrap();
        assert!(!body.stream);
        assert!(body.body.get("max_tokens").is_none());
        assert!(body.body.get("enable_think").is_none());
    }

    #[test]
    fn translates_generate_and_detects_loading() {
        let body =
            generate_request(&json!({ "model": "m", "system": "S", "prompt": "P" })).unwrap();
        assert_eq!(body.body["messages"][0]["content"], "S");
        assert_eq!(body.body["messages"][1]["content"], "P");
        assert!(is_load_only(&json!({ "model": "m" })));
        assert!(is_load_only(&json!({ "model": "m", "prompt": "" })));
        assert!(!is_load_only(&json!({ "model": "m", "prompt": "Hi" })));
    }

    #[test]
    fn requires_a_model() {
        assert!(chat_request(&json!({ "messages": [] })).is_err());
    }

    #[test]
    fn turns_events_into_ollama_chunks() {
        let mut answer = Answer::new("m", Mode::Chat);
        let chunks = answer.feed(
            b"data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"Hmm\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"Hel",
        );
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0]["message"]["thinking"], "Hmm");
        let chunks = answer.feed(
            b"lo\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2},\"timings\":{\"predicted_ms\":50.0}}\n\ndata: [DONE]\n\n",
        );
        assert_eq!(chunks[0]["message"]["content"], "Hello");
        assert_eq!(chunks[0]["done"], false);

        let done = answer.done(120);
        assert_eq!(done["done"], true);
        assert_eq!(done["message"]["content"], "");
        assert_eq!(done["done_reason"], "stop");
        assert_eq!(done["eval_count"], 2);
        assert_eq!(done["eval_duration"], 50_000_000);
        assert_eq!(done["total_duration"], 120_000_000);

        let whole = answer.whole(120);
        assert_eq!(whole["message"]["content"], "Hello");
        assert_eq!(whole["message"]["thinking"], "Hmm");
    }

    #[test]
    fn collects_tool_calls_and_errors() {
        let mut answer = Answer::new("m", Mode::Chat);
        answer.feed(b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"weather\",\"arguments\":\"{\\\"city\\\":\"}}]}}]}\n");
        answer.feed(b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"Paris\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n");
        let done = answer.whole(1);
        assert_eq!(
            done["message"]["tool_calls"][0],
            json!({ "function": { "name": "weather", "arguments": { "city": "Paris" } } })
        );

        let mut failing = Answer::new("m", Mode::Generate);
        failing.feed(b"data:{\"error\":{\"message\":\"model not found\"}}\n");
        assert_eq!(failing.error(), Some("model not found"));
    }

    #[test]
    fn generate_answers_with_response() {
        let mut answer = Answer::new("m", Mode::Generate);
        let chunks = answer.feed(b"data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n");
        assert_eq!(chunks[0]["response"], "Hi");
        assert!(chunks[0].get("message").is_none());
    }

    #[test]
    fn formats_rfc3339() {
        assert_eq!(rfc3339(0, 0), "1970-01-01T00:00:00.000000000Z");
        assert_eq!(rfc3339(1_791_504_000, 5), "2026-10-09T00:00:00.000000005Z");
        assert_eq!(rfc3339(951_782_400, 0), "2000-02-29T00:00:00.000000000Z");
    }
}
