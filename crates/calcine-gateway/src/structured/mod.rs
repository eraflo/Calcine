//! Replies of a required shape, for every model, the NPU included:
//! structured output (`response_format`) and forced tool calls
//! (`tool_choice`).
//!
//! GenieX 0.8 ignores both in API requests, so Calcine does it: it asks for
//! JSON (with the schema, if any) or for a tool call, checks the reply, and
//! asks the model to fix it when it doesn't match, a few times at most. The
//! client gets the usual OpenAI shape, streamed or not.

mod schema;
pub mod tools;

use std::sync::Arc;

use axum::body::Bytes;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};

use crate::caller::Caller;
use crate::error::{api_error, upstream_message};
use crate::proxy::{self, elapsed_ms};
use crate::state::AppState;

pub use schema::{extract, filled, validate};

/// Tries after the first answer.
const RETRIES: usize = 2;

/// Introduces the filled-in example in instructions (the mock backend looks
/// for it). The example's strings are `<placeholders>`.
pub const SHAPE: &str = "Use this shape, replacing each <placeholder> with a real value:";

/// The JSON the client asked for.
#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    /// `{"type": "json_object"}`: any JSON object.
    Object,
    /// `{"type": "json_schema", "json_schema": {"schema": …}}`.
    Schema { name: Option<String>, schema: Value },
}

/// What a reply must be.
#[derive(Debug, Clone, PartialEq)]
pub enum Goal {
    /// JSON content in this format.
    Json(Format),
    /// A call to one of the tools.
    Tool(tools::Forced),
}

/// The structured format a chat request asks for, if any.
pub fn requested(body: &Value) -> Option<Format> {
    let format = body.get("response_format")?;
    match format.get("type").and_then(Value::as_str)? {
        "json_object" => Some(Format::Object),
        "json_schema" => {
            let spec = format.get("json_schema")?;
            Some(Format::Schema {
                name: spec.get("name").and_then(Value::as_str).map(str::to_owned),
                schema: spec.get("schema").cloned().unwrap_or(json!({})),
            })
        }
        _ => None,
    }
}

/// What a chat request requires of the reply, if anything: a forced tool
/// call comes first. An error when `tool_choice` doesn't make sense.
pub fn goal(body: &Value) -> Result<Option<Goal>, String> {
    if let Some(forced) = tools::forced(body)? {
        return Ok(Some(Goal::Tool(forced)));
    }
    Ok(requested(body).map(Goal::Json))
}

/// The instruction added to the system prompt.
pub fn instruction(format: &Format) -> String {
    let base = "Reply with a single valid JSON value and nothing else: no explanation, \
                no Markdown, no code fence.";
    match format {
        Format::Object => format!("{base} The value must be a JSON object."),
        // Small models follow a filled-in example better than a schema.
        Format::Schema { name, schema } => format!(
            "{base} {SHAPE}\n{}\n\nIt must match this JSON Schema{}:\n{}",
            schema::example(schema),
            name.as_ref()
                .map(|name| format!(" (\"{name}\")"))
                .unwrap_or_default(),
            serde_json::to_string_pretty(schema).unwrap_or_default()
        ),
    }
}

/// Why `value` isn't acceptable for `format`, if it isn't.
pub fn check(value: &Value, format: &Format) -> Result<(), String> {
    match format {
        Format::Object if !value.is_object() => Err("the reply must be a JSON object".into()),
        Format::Object => Ok(()),
        Format::Schema { schema, .. } => {
            validate(value, schema).and_then(|()| filled(value, schema))
        }
    }
}

impl Goal {
    /// The request GenieX gets: the instruction in the system prompt, not
    /// streamed, without what GenieX would ignore anyway.
    pub fn upstream_request(&self, body: &Value) -> Value {
        let mut request = body.clone();
        let Some(object) = request.as_object_mut() else {
            return request;
        };
        for key in [
            "response_format",
            "tool_choice",
            "stream_options",
            "parallel_tool_calls",
        ] {
            object.remove(key);
        }
        object.insert("stream".into(), json!(false));
        // Reasoning goes to `reasoning_content`, out of the reply.
        object.insert("reasoning_format".into(), json!("deepseek"));
        let instruction = match self {
            Self::Json(format) => instruction(format),
            // The call is asked for as JSON, in the instruction.
            Self::Tool(forced) => {
                object.remove("tools");
                tools::instruction(forced)
            }
        };
        let mut messages = object
            .get_mut("messages")
            .and_then(Value::as_array_mut)
            .map(std::mem::take)
            .unwrap_or_default();
        match messages.first_mut() {
            Some(first) if first.get("role").and_then(Value::as_str) == Some("system") => {
                let current = first.get("content").and_then(Value::as_str).unwrap_or("");
                first["content"] = json!(format!("{current}\n\n{instruction}").trim().to_owned());
            }
            _ => messages.insert(0, json!({ "role": "system", "content": instruction })),
        }
        object.insert("messages".into(), Value::Array(messages));
        request
    }

    /// The reply's message as the client gets it, or why it doesn't do.
    fn accept(&self, message: &Value) -> Result<Value, String> {
        match self {
            Self::Json(format) => {
                let text = message.get("content").and_then(Value::as_str).unwrap_or("");
                let value = extract(text).ok_or_else(|| "the reply isn't JSON".to_owned())?;
                check(&value, format)?;
                let mut accepted = message.clone();
                accepted["content"] = json!(value.to_string());
                if let Some(object) = accepted.as_object_mut() {
                    object.remove("tool_calls");
                }
                Ok(accepted)
            }
            Self::Tool(forced) => tools::check(message, forced),
        }
    }

    /// What to ask after showing the model what's wrong.
    const fn fix_hint(&self) -> &'static str {
        match self {
            Self::Json(_) => "Reply again with only the corrected JSON.",
            Self::Tool(_) => "Reply again with only the corrected JSON call.",
        }
    }

    const fn failure(&self) -> &'static str {
        match self {
            Self::Json(_) => "the model didn't produce valid JSON",
            Self::Tool(_) => "the model didn't call a tool as required",
        }
    }
}

/// How the client wants the answer delivered.
#[derive(Debug, Clone, Copy)]
struct Delivery {
    stream: bool,
    include_usage: bool,
}

/// Why the reply couldn't be had, to answer in the client's error format.
#[derive(Debug)]
pub struct Failure {
    pub status: StatusCode,
    pub kind: &'static str,
    pub message: String,
}

/// Answer a chat request that sets a goal.
pub async fn complete(app: Arc<AppState>, caller: Caller, body: Value, goal: Goal) -> Response {
    let delivery = Delivery {
        stream: body.get("stream").and_then(Value::as_bool).unwrap_or(false),
        include_usage: body
            .pointer("/stream_options/include_usage")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };
    match generate(
        &app,
        &caller,
        &body,
        &goal,
        "/v1/chat/completions",
        delivery.stream,
    )
    .await
    {
        Ok(answer) => deliver(&answer, delivery),
        Err(failure) => api_error(failure.status, failure.kind, &failure.message),
    }
}

/// Ask GenieX until the reply meets `goal`: the completion, with the
/// checked message. The request log shows `logged_path`.
pub async fn generate(
    app: &Arc<AppState>,
    caller: &Caller,
    body: &Value,
    goal: &Goal,
    logged_path: &'static str,
    client_streams: bool,
) -> Result<Value, Failure> {
    let mut request = goal.upstream_request(body);
    let bytes = Bytes::from(request.to_string());
    let (exchange, response) = proxy::send(
        app,
        caller,
        logged_path,
        "/v1/chat/completions",
        &request,
        bytes,
    )
    .await
    .map_err(|refused| Failure {
        status: refused.status,
        kind: refused.kind,
        message: refused.message,
    })?;
    exchange
        .log
        .update(exchange.id, |entry| entry.stream = client_streams);

    let mut response = Some(response);
    let mut usage = (0_u64, 0_u64);
    let mut last_error = String::new();
    for attempt in 0..=RETRIES {
        let reply = match response.take() {
            Some(first) => Ok(first),
            None => retry(app, &request).await,
        };
        let completion = read_completion(reply).await.map_err(|(status, message)| {
            exchange.log_failure(status, &message);
            Failure {
                status,
                kind: "server_error",
                message,
            }
        })?;
        usage.0 += token_count(&completion, "prompt_tokens");
        usage.1 += token_count(&completion, "completion_tokens");
        let message = completion
            .pointer("/choices/0/message")
            .cloned()
            .unwrap_or_default();
        match goal.accept(&message) {
            Ok(accepted) => {
                exchange.log.update(exchange.id, |entry| {
                    entry.status = Some(200);
                    entry.duration_ms = Some(elapsed_ms(exchange.started));
                    entry.prompt_tokens = Some(usage.0);
                    entry.completion_tokens = Some(usage.1);
                    entry.tokens_per_second = completion
                        .pointer("/timings/predicted_per_second")
                        .and_then(Value::as_f64);
                });
                return Ok(finished(completion, accepted, usage));
            }
            Err(reason) => {
                tracing::info!(attempt, %reason, "the reply didn't meet the goal, asking again");
                ask_to_fix(&mut request, &message, &reason, goal.fix_hint());
                last_error = reason;
            }
        }
    }
    let message = format!(
        "{} after {} tries: {last_error}",
        goal.failure(),
        RETRIES + 1
    );
    exchange.log_failure(StatusCode::UNPROCESSABLE_ENTITY, &message);
    Err(Failure {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        kind: "invalid_response",
        message,
    })
}

/// The completion GenieX returned, or the error to answer with.
async fn read_completion(
    reply: Result<reqwest::Response, String>,
) -> Result<Value, (StatusCode, String)> {
    let reply = reply.map_err(|message| (StatusCode::BAD_GATEWAY, message))?;
    let status = reply.status();
    let bytes = reply.bytes().await.unwrap_or_default();
    if !status.is_success() {
        return Err((status, upstream_message(&bytes)));
    }
    Ok(serde_json::from_slice(&bytes).unwrap_or_default())
}

fn token_count(completion: &Value, field: &str) -> u64 {
    completion
        .get("usage")
        .and_then(|usage| usage.get(field))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

/// Show the model its reply and what's wrong with it.
fn ask_to_fix(request: &mut Value, message: &Value, reason: &str, hint: &str) {
    let mut reply = json!({
        "role": "assistant",
        "content": message.get("content").and_then(Value::as_str).unwrap_or(""),
    });
    if let Some(calls) = message
        .get("tool_calls")
        .filter(|calls| calls.as_array().is_some_and(|calls| !calls.is_empty()))
    {
        reply["tool_calls"] = calls.clone();
    }
    if let Some(messages) = request["messages"].as_array_mut() {
        messages.push(reply);
        messages.push(json!({
            "role": "user",
            "content": format!("That reply isn't valid: {reason}. {hint}"),
        }));
    }
}

fn deliver(answer: &Value, delivery: Delivery) -> Response {
    if delivery.stream {
        as_stream(answer, delivery.include_usage)
    } else {
        (
            [(header::CONTENT_TYPE, "application/json")],
            answer.to_string(),
        )
            .into_response()
    }
}

/// Ask again, within the same queue turn.
async fn retry(app: &AppState, request: &Value) -> Result<reqwest::Response, String> {
    let upstream = app
        .services
        .server
        .ensure_running()
        .await
        .map_err(|err| format!("GenieX couldn't start: {err}"))?;
    app.http
        .post(format!("{upstream}/v1/chat/completions"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(request.to_string())
        .send()
        .await
        .map_err(|err| format!("couldn't reach GenieX: {err}"))
}

/// The last completion, with the checked message and the usage of every
/// try.
fn finished(mut completion: Value, message: Value, usage: (u64, u64)) -> Value {
    let calls_tools = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .is_some_and(|calls| !calls.is_empty());
    if let Some(choice) = completion.pointer_mut("/choices/0") {
        choice["message"] = message;
        choice["finish_reason"] = json!(if calls_tools { "tool_calls" } else { "stop" });
    }
    completion["usage"] = json!({
        "prompt_tokens": usage.0,
        "completion_tokens": usage.1,
        "total_tokens": usage.0 + usage.1,
    });
    completion
}

/// The answer as server-sent events, for clients that asked for a stream.
fn as_stream(answer: &Value, include_usage: bool) -> Response {
    let base = |choices: Value| {
        let mut chunk = Map::new();
        for key in ["id", "created", "model"] {
            if let Some(value) = answer.get(key) {
                chunk.insert(key.into(), value.clone());
            }
        }
        chunk.insert("object".into(), json!("chat.completion.chunk"));
        chunk.insert("choices".into(), choices);
        Value::Object(chunk)
    };
    let mut events = vec![
        base(json!([{ "index": 0, "delta": delta(answer), "finish_reason": null }])),
        base(json!([{
            "index": 0,
            "delta": {},
            "finish_reason": answer.pointer("/choices/0/finish_reason").cloned().unwrap_or(json!("stop")),
        }])),
    ];
    if include_usage {
        let mut last = base(json!([]));
        last["usage"] = answer["usage"].clone();
        if let Some(timings) = answer.get("timings") {
            last["timings"] = timings.clone();
        }
        events.push(last);
    }
    let mut body = String::new();
    for event in &events {
        body.push_str("data: ");
        body.push_str(&event.to_string());
        body.push_str("\n\n");
    }
    body.push_str("data: [DONE]\n\n");
    (
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response()
}

/// The whole message as one delta; tool calls get their stream index.
fn delta(answer: &Value) -> Value {
    let message = answer
        .pointer("/choices/0/message")
        .cloned()
        .unwrap_or_default();
    let mut delta = json!({ "role": "assistant", "content": message["content"] });
    if let Some(reasoning) = message
        .get("reasoning_content")
        .filter(|value| value.is_string())
    {
        delta["reasoning_content"] = reasoning.clone();
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        let calls: Vec<Value> = calls
            .iter()
            .enumerate()
            .map(|(index, call)| {
                let mut call = call.clone();
                call["index"] = json!(index);
                call
            })
            .collect();
        delta["tool_calls"] = Value::Array(calls);
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_requested_format() {
        assert_eq!(
            requested(&json!({ "response_format": { "type": "json_object" } })),
            Some(Format::Object)
        );
        let schema = json!({ "type": "object" });
        assert_eq!(
            requested(
                &json!({ "response_format": { "type": "json_schema", "json_schema": { "name": "p", "schema": schema } } })
            ),
            Some(Format::Schema {
                name: Some("p".into()),
                schema
            })
        );
        assert_eq!(
            requested(&json!({ "response_format": { "type": "text" } })),
            None
        );
        assert_eq!(requested(&json!({})), None);
    }

    #[test]
    fn a_forced_call_comes_before_the_format() {
        let body = json!({
            "tools": [{ "type": "function", "function": { "name": "f" } }],
            "tool_choice": "required",
            "response_format": { "type": "json_object" },
        });
        assert!(matches!(goal(&body), Ok(Some(Goal::Tool(_)))));
        assert!(matches!(
            goal(&json!({ "response_format": { "type": "json_object" } })),
            Ok(Some(Goal::Json(Format::Object)))
        ));
        assert_eq!(goal(&json!({})), Ok(None));
    }

    #[test]
    fn adds_the_instruction_to_the_system_prompt() {
        let body = json!({
            "model": "m",
            "stream": true,
            "stream_options": { "include_usage": true },
            "response_format": { "type": "json_object" },
            "messages": [{ "role": "system", "content": "Be brief." }, { "role": "user", "content": "Hi" }],
        });
        let request = Goal::Json(Format::Object).upstream_request(&body);
        assert_eq!(request["stream"], false);
        assert!(request.get("response_format").is_none());
        let system = request["messages"][0]["content"].as_str().unwrap();
        assert!(system.starts_with("Be brief.\n\nReply with a single valid JSON value"));
        assert_eq!(request["messages"][1]["content"], "Hi");

        let bare = Goal::Json(Format::Schema {
            name: None,
            schema: json!({ "type": "array" }),
        })
        .upstream_request(&json!({ "messages": [{ "role": "user", "content": "Hi" }] }));
        assert_eq!(bare["messages"][0]["role"], "system");
        assert!(
            bare["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("\"array\"")
        );
    }

    #[test]
    fn offers_only_the_named_tool() {
        let body = json!({
            "messages": [{ "role": "user", "content": "Hi" }],
            "tools": [
                { "type": "function", "function": { "name": "a" } },
                { "type": "function", "function": { "name": "b" } },
            ],
            "tool_choice": { "type": "function", "function": { "name": "b" } },
        });
        let Ok(Some(goal)) = goal(&body) else {
            panic!("expected a forced call")
        };
        let request = goal.upstream_request(&body);
        assert!(request.get("tool_choice").is_none() && request.get("tools").is_none());
        let system = request["messages"][0]["content"].as_str().unwrap();
        assert!(system.starts_with("Call the b tool now"));
        assert!(system.contains("- b") && !system.contains("- a"));
    }

    #[test]
    fn objects_must_be_objects() {
        assert!(check(&json!({ "a": 1 }), &Format::Object).is_ok());
        assert!(check(&json!([1]), &Format::Object).is_err());
    }

    #[test]
    fn streams_the_checked_answer() {
        let completion = json!({ "id": "x", "model": "m", "choices": [{ "message": { "role": "assistant", "content": "junk" } }] });
        let accepted = Goal::Json(Format::Object)
            .accept(&json!({ "role": "assistant", "content": "Here: {\"a\": 1}" }))
            .unwrap();
        let answer = finished(completion, accepted, (10, 5));
        assert_eq!(answer["choices"][0]["message"]["content"], "{\"a\":1}");
        assert_eq!(answer["choices"][0]["finish_reason"], "stop");
        assert_eq!(answer["usage"]["total_tokens"], 15);
        let _ = as_stream(&answer, true);
    }

    #[test]
    fn streams_tool_calls_with_their_index() {
        let call = json!({ "id": "c", "type": "function", "function": { "name": "f", "arguments": "{}" } });
        let answer = finished(
            json!({ "choices": [{ "message": {} }] }),
            json!({ "role": "assistant", "content": null, "tool_calls": [call] }),
            (1, 1),
        );
        assert_eq!(answer["choices"][0]["finish_reason"], "tool_calls");
        assert_eq!(delta(&answer)["tool_calls"][0]["index"], 0);
    }
}
