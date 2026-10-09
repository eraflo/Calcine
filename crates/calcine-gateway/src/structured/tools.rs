//! Forced tool calls (`tool_choice`) for every model, the NPU included.
//!
//! GenieX 0.8 gives the model its `tools` and returns the calls it makes,
//! but ignores `tool_choice`. Calcine enforces it: `none` takes the tools
//! away, `required` or a named function asks for the call as JSON and checks
//! it: a tool from the list, with arguments that match its parameters.
//!
//! The call is asked for as JSON, with a filled-in example, rather than
//! through the model's own tool format: on the NPU, Qwen3 4B skips tools it
//! thinks it doesn't need and Qwen3 0.6B ignores the instruction, while both
//! write the JSON reliably.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::SHAPE;
use super::schema::{example, extract, filled, validate};

/// A call the client requires.
#[derive(Debug, Clone, PartialEq)]
pub struct Forced {
    /// The function to call, or `None` for any of `tools`.
    pub function: Option<String>,
    /// The tools the model may call.
    pub tools: Vec<Value>,
}

/// Whether the request turns tools off (`tool_choice: "none"`).
pub fn refused(body: &Value) -> bool {
    body.get("tool_choice").and_then(Value::as_str) == Some("none")
}

/// The request without its tools, for `tool_choice: "none"`.
pub fn without_tools(body: &Value) -> Value {
    let mut request = body.clone();
    if let Some(object) = request.as_object_mut() {
        for key in ["tools", "tool_choice", "parallel_tool_calls"] {
            object.remove(key);
        }
    }
    request
}

/// The call a chat request requires, if any; an error when `tool_choice`
/// names a function that isn't in `tools`.
pub fn forced(body: &Value) -> Result<Option<Forced>, String> {
    let tools: Vec<Value> = body
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let function = match body.get("tool_choice") {
        Some(Value::String(choice)) if choice == "required" => None,
        Some(choice @ Value::Object(_)) => {
            let Some(name) = choice
                .pointer("/function/name")
                .or_else(|| choice.get("name"))
                .and_then(Value::as_str)
            else {
                return Err("tool_choice needs a function name".into());
            };
            if !tools.iter().any(|tool| tool_name(tool) == Some(name)) {
                return Err(format!("tool_choice names {name}, which isn't in tools"));
            }
            Some(name.to_owned())
        }
        _ => return Ok(None),
    };
    if tools.is_empty() {
        return Err("tool_choice requires a call but no tools were given".into());
    }
    Ok(Some(Forced { function, tools }))
}

fn tool_name(tool: &Value) -> Option<&str> {
    tool.pointer("/function/name").and_then(Value::as_str)
}

fn parameters(tool: &Value) -> Value {
    tool.pointer("/function/parameters")
        .cloned()
        .unwrap_or(json!({ "type": "object" }))
}

/// The instruction added to the system prompt (GenieX gets no `tools`).
pub fn instruction(forced: &Forced) -> String {
    let offered = offered(forced);
    let lead = match &forced.function {
        Some(name) => format!("Call the {name} tool now, even if you could answer without it."),
        None => "Call one of the tools below now, even if you could answer without it.".to_owned(),
    };
    // The name first, as models write calls.
    let shape = offered.first().map_or_else(String::new, |tool| {
        format!(
            r#"{{"name": {}, "arguments": {}}}"#,
            json!(tool_name(tool)),
            example(&parameters(tool))
        )
    });
    let list: Vec<String> = offered
        .iter()
        .map(|tool| {
            let description = tool
                .pointer("/function/description")
                .and_then(Value::as_str)
                .map(|text| format!(": {text}"))
                .unwrap_or_default();
            format!(
                "- {}{description}\n  Arguments (JSON Schema): {}",
                tool_name(tool).unwrap_or(""),
                parameters(tool)
            )
        })
        .collect();
    format!(
        "{lead} Reply with only the call, as a single valid JSON value and nothing else: no \
         explanation, no Markdown, no code fence. {SHAPE}\n{shape}\n\nThe tools:\n{}",
        list.join("\n")
    )
}

/// The tools the model may call: only the named function, when there is one.
fn offered(forced: &Forced) -> Vec<&Value> {
    forced
        .tools
        .iter()
        .filter(|tool| {
            forced
                .function
                .as_deref()
                .is_none_or(|name| tool_name(tool) == Some(name))
        })
        .collect()
}

/// The calls in a reply: GenieX's `tool_calls`, or the JSON in its content
/// (one call, or an array of them).
fn calls_in(message: &Value) -> Vec<Value> {
    if let Some(calls) = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .filter(|calls| !calls.is_empty())
    {
        return calls.clone();
    }
    let written = message
        .get("content")
        .and_then(Value::as_str)
        .and_then(extract);
    let written = match written {
        Some(Value::Array(calls)) => calls,
        Some(call @ Value::Object(_)) => vec![call],
        _ => return Vec::new(),
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_micros());
    written
        .iter()
        .filter_map(|call| {
            let name = call.get("name")?.as_str()?;
            let arguments = match call.get("arguments") {
                Some(Value::String(text)) => text.clone(),
                Some(value) => value.to_string(),
                None => "{}".to_owned(),
            };
            Some((name.to_owned(), arguments))
        })
        .enumerate()
        .map(|(index, (name, arguments))| {
            json!({
                "id": format!("call_{stamp:x}{index}"),
                "type": "function",
                "function": { "name": name, "arguments": arguments },
            })
        })
        .collect()
}

/// The reply's message with its checked calls, or why it doesn't do.
pub fn check(message: &Value, forced: &Forced) -> Result<Value, String> {
    let calls = calls_in(message);
    if calls.is_empty() {
        return Err(match &forced.function {
            Some(name) => format!("the {name} tool wasn't called"),
            None => "no tool was called".to_owned(),
        });
    }
    for call in &calls {
        let name = call
            .pointer("/function/name")
            .and_then(Value::as_str)
            .unwrap_or("");
        if let Some(wanted) = &forced.function
            && name != wanted
        {
            return Err(format!("{name} was called instead of {wanted}"));
        }
        let tool = forced
            .tools
            .iter()
            .find(|tool| tool_name(tool) == Some(name))
            .ok_or_else(|| format!("{name} isn't one of the tools"))?;
        let arguments = match call.pointer("/function/arguments") {
            Some(Value::String(text)) if text.trim().is_empty() => json!({}),
            Some(Value::String(text)) => serde_json::from_str(text)
                .map_err(|_| format!("the arguments of {name} aren't valid JSON"))?,
            Some(value) => value.clone(),
            None => json!({}),
        };
        validate(&arguments, &parameters(tool))
            .and_then(|()| filled(&arguments, &parameters(tool)))
            .map_err(|reason| format!("{name}: {reason}"))?;
    }
    let mut checked = json!({ "role": "assistant", "content": null, "tool_calls": calls });
    if let Some(reasoning) = message
        .get("reasoning_content")
        .filter(|reasoning| reasoning.as_str().is_some_and(|text| !text.is_empty()))
    {
        checked["reasoning_content"] = reasoning.clone();
    }
    Ok(checked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weather() -> Value {
        json!({ "type": "function", "function": { "name": "get_weather", "parameters": {
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"],
        } } })
    }

    fn time() -> Value {
        json!({ "type": "function", "function": { "name": "get_time" } })
    }

    fn call(name: &str, arguments: &str) -> Value {
        json!({ "role": "assistant", "content": "", "tool_calls": [
            { "id": "call_1", "type": "function", "function": { "name": name, "arguments": arguments } },
        ] })
    }

    #[test]
    fn reads_tool_choice() {
        let tools = json!([weather(), time()]);
        assert_eq!(forced(&json!({ "tools": tools })), Ok(None));
        assert_eq!(
            forced(&json!({ "tools": tools, "tool_choice": "auto" })),
            Ok(None)
        );
        assert_eq!(
            forced(&json!({ "tools": tools, "tool_choice": "required" }))
                .unwrap()
                .unwrap()
                .function,
            None
        );
        let named = json!({ "tools": tools, "tool_choice": { "type": "function", "function": { "name": "get_time" } } });
        let named = forced(&named).unwrap().unwrap();
        assert_eq!(named.function.as_deref(), Some("get_time"));
        assert_eq!(offered(&named), vec![&time()]);

        assert!(
            forced(&json!({ "tools": tools, "tool_choice": { "function": { "name": "nope" } } }))
                .is_err()
        );
        assert!(forced(&json!({ "tool_choice": "required" })).is_err());
    }

    #[test]
    fn shows_the_call_to_write() {
        let any = Forced {
            function: None,
            tools: vec![weather(), time()],
        };
        let instruction = instruction(&any);
        assert!(instruction.contains(r#"{"name": "get_weather", "arguments": {"city":"<city>"}}"#));
        assert!(instruction.contains("- get_weather\n  Arguments (JSON Schema):"));
        assert!(instruction.contains("- get_time"));
    }

    #[test]
    fn none_takes_the_tools_away() {
        let body = json!({ "model": "m", "tools": [weather()], "tool_choice": "none" });
        assert!(refused(&body));
        assert_eq!(without_tools(&body), json!({ "model": "m" }));
        assert!(!refused(&json!({ "tool_choice": "auto" })));
    }

    #[test]
    fn checks_the_calls() {
        let any = Forced {
            function: None,
            tools: vec![weather(), time()],
        };
        let checked = check(&call("get_weather", r#"{"city": "Paris"}"#), &any).unwrap();
        assert_eq!(checked["content"], Value::Null);
        assert_eq!(checked["tool_calls"][0]["function"]["name"], "get_weather");
        assert!(check(&call("get_time", ""), &any).is_ok());

        assert_eq!(
            check(&json!({ "content": "Sunny." }), &any),
            Err("no tool was called".into())
        );
        assert_eq!(
            check(&call("get_weather", "{}"), &any),
            Err("get_weather: $.city is missing".into())
        );
        assert_eq!(
            check(&call("get_weather", "{city"), &any),
            Err("the arguments of get_weather aren't valid JSON".into())
        );
        assert_eq!(
            check(&call("get_weather", r#"{"city": "<city>"}"#), &any),
            Err(
                "get_weather: $.city is still the placeholder \"<city>\", write a real value"
                    .into()
            )
        );
        assert_eq!(
            check(&call("search", "{}"), &any),
            Err("search isn't one of the tools".into())
        );

        // Calls written as JSON in the content, one or several.
        let written = json!({ "content": "```json\n{\"name\": \"get_weather\", \"arguments\": {\"city\": \"Paris\"}}\n```" });
        let checked = check(&written, &any).unwrap();
        assert_eq!(checked["tool_calls"][0]["type"], "function");
        assert_eq!(
            checked["tool_calls"][0]["function"]["arguments"],
            "{\"city\":\"Paris\"}"
        );
        let several = json!({ "content": r#"[{"name": "get_time", "arguments": {}}, {"name": "get_time"}]"# });
        let checked = check(&several, &any).unwrap();
        assert_eq!(checked["tool_calls"].as_array().unwrap().len(), 2);
        assert_ne!(
            checked["tool_calls"][0]["id"],
            checked["tool_calls"][1]["id"]
        );

        let named = Forced {
            function: Some("get_time".into()),
            tools: vec![weather(), time()],
        };
        assert_eq!(
            check(&call("get_weather", r#"{"city": "Paris"}"#), &named),
            Err("get_weather was called instead of get_time".into())
        );
    }
}
