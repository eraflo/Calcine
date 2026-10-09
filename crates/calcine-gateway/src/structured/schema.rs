//! Reading JSON out of a model's reply and checking it against a JSON Schema
//! subset: `type`, `enum`, `const`, `properties`, `required`,
//! `additionalProperties`, `items`, `minItems`/`maxItems`,
//! `minLength`/`maxLength`, `minimum`/`maximum`, `anyOf`/`oneOf`. Other
//! keywords are accepted without being checked.

use serde_json::Value;

/// The JSON in a reply: the whole text, a fenced block, or the first object
/// or array in it.
pub fn extract(text: &str) -> Option<Value> {
    let text = text.trim();
    if let Ok(value) = serde_json::from_str(text) {
        return Some(value);
    }
    if let Some(fenced) = fenced_block(text)
        && let Ok(value) = serde_json::from_str(fenced.trim())
    {
        return Some(value);
    }
    let start = text.find(['{', '['])?;
    let close = if text[start..].starts_with('{') {
        '}'
    } else {
        ']'
    };
    let end = text.rfind(close)?;
    (end > start)
        .then(|| serde_json::from_str(&text[start..=end]).ok())
        .flatten()
}

fn fenced_block(text: &str) -> Option<&str> {
    let start = text.find("```")?;
    let after = &text[start + 3..];
    let body_start = after.find('\n')? + 1;
    let body = &after[body_start..];
    let end = body.find("```")?;
    Some(&body[..end])
}

/// A placeholder value with the schema's shape: every property, one item
/// per array, the first allowed value of an enum. Strings are named after
/// what they hold, `"<city>"`: small models replace those, while they copy
/// `"..."` as it is.
pub fn example(schema: &Value) -> Value {
    named_example(schema, "value")
}

fn named_example(schema: &Value, label: &str) -> Value {
    let Some(schema) = schema.as_object() else {
        return Value::Null;
    };
    if let Some(first) = schema
        .get("enum")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
    {
        return first.clone();
    }
    if let Some(constant) = schema.get("const") {
        return constant.clone();
    }
    if let Some(first) = ["anyOf", "oneOf"]
        .iter()
        .find_map(|keyword| schema.get(*keyword).and_then(Value::as_array))
        .and_then(|options| options.first())
    {
        return named_example(first, label);
    }
    let kind = match schema.get("type") {
        Some(Value::String(kind)) => kind.as_str(),
        Some(Value::Array(kinds)) => kinds
            .iter()
            .filter_map(Value::as_str)
            .find(|kind| *kind != "null")
            .unwrap_or("null"),
        _ if schema.contains_key("properties") => "object",
        _ if schema.contains_key("items") => "array",
        _ => "string",
    };
    match kind {
        "object" => Value::Object(
            schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| {
                    properties
                        .iter()
                        .map(|(key, property)| (key.clone(), named_example(property, key)))
                        .collect()
                })
                .unwrap_or_default(),
        ),
        "array" => Value::Array(vec![named_example(
            schema.get("items").unwrap_or(&Value::Null),
            &format!("{label} item"),
        )]),
        "integer" | "number" => Value::from(0),
        "boolean" => Value::Bool(false),
        "null" => Value::Null,
        _ => Value::String(format!("<{label}>")),
    }
}

/// `Ok` when `value` matches `schema`, else where and why it doesn't.
pub fn validate(value: &Value, schema: &Value) -> Result<(), String> {
    check(value, schema, "$")
}

/// `Ok` when no string in `value` is a placeholder from `schema`'s example:
/// small models sometimes copy the example as it is.
pub fn filled(value: &Value, schema: &Value) -> Result<(), String> {
    let mut placeholders = Vec::new();
    strings(&example(schema), &mut placeholders);
    copied(value, &placeholders, "$").map_or(Ok(()), |(path, placeholder)| {
        Err(format!(
            "{path} is still the placeholder \"{placeholder}\", write a real value"
        ))
    })
}

fn strings(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::String(text) if text.starts_with('<') => found.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| strings(item, found)),
        Value::Object(fields) => fields.values().for_each(|field| strings(field, found)),
        _ => {}
    }
}

fn copied(value: &Value, placeholders: &[String], path: &str) -> Option<(String, String)> {
    match value {
        Value::String(text) if placeholders.contains(text) => Some((path.to_owned(), text.clone())),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .find_map(|(index, item)| copied(item, placeholders, &format!("{path}[{index}]"))),
        Value::Object(fields) => fields
            .iter()
            .find_map(|(key, field)| copied(field, placeholders, &format!("{path}.{key}"))),
        _ => None,
    }
}

type Schema = serde_json::Map<String, Value>;

fn check(value: &Value, schema: &Value, path: &str) -> Result<(), String> {
    let Value::Object(schema) = schema else {
        // `true` and anything that isn't a schema object allow everything.
        return if *schema == Value::Bool(false) {
            Err(format!("{path} isn't allowed"))
        } else {
            Ok(())
        };
    };
    check_shape(value, schema, path)?;
    match value {
        Value::Object(object) => check_object(object, schema, path),
        Value::Array(items) => check_array(items, schema, path),
        Value::String(text) => check_length(text.chars().count(), schema, path),
        Value::Number(number) => check_range(number.as_f64().unwrap_or(0.0), schema, path),
        Value::Null | Value::Bool(_) => Ok(()),
    }
}

/// `anyOf`/`oneOf`, `enum`, `const` and `type`.
fn check_shape(value: &Value, schema: &Schema, path: &str) -> Result<(), String> {
    for keyword in ["anyOf", "oneOf"] {
        if let Some(options) = schema.get(keyword).and_then(Value::as_array)
            && !options
                .iter()
                .any(|option| check(value, option, path).is_ok())
        {
            return Err(format!("{path} matches none of the allowed shapes"));
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array)
        && !allowed.contains(value)
    {
        return Err(format!(
            "{path} must be one of {}",
            Value::Array(allowed.clone())
        ));
    }
    if let Some(constant) = schema.get("const")
        && constant != value
    {
        return Err(format!("{path} must be {constant}"));
    }
    if let Some(kind) = schema.get("type") {
        let kinds: Vec<&str> = match kind {
            Value::String(kind) => vec![kind.as_str()],
            Value::Array(kinds) => kinds.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if !kinds.is_empty() && !kinds.iter().any(|kind| is_type(value, kind)) {
            return Err(format!(
                "{path} must be {}, not {}",
                kinds.join(" or "),
                type_of(value)
            ));
        }
    }
    Ok(())
}

fn check_object(
    object: &serde_json::Map<String, Value>,
    schema: &Schema,
    path: &str,
) -> Result<(), String> {
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            if !object.contains_key(key) {
                return Err(format!("{path}.{key} is missing"));
            }
        }
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    for (key, item) in object {
        let item_path = format!("{path}.{key}");
        match properties.and_then(|properties| properties.get(key)) {
            Some(property) => check(item, property, &item_path)?,
            None => match schema.get("additionalProperties") {
                Some(Value::Bool(false)) => {
                    return Err(format!("{item_path} isn't an expected field"));
                }
                Some(extra @ Value::Object(_)) => check(item, extra, &item_path)?,
                _ => {}
            },
        }
    }
    Ok(())
}

fn check_array(items: &[Value], schema: &Schema, path: &str) -> Result<(), String> {
    let length = items.len() as u64;
    if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
        && length < min
    {
        return Err(format!("{path} needs at least {min} items"));
    }
    if let Some(max) = schema.get("maxItems").and_then(Value::as_u64)
        && length > max
    {
        return Err(format!("{path} allows at most {max} items"));
    }
    if let Some(item_schema) = schema.get("items") {
        for (index, item) in items.iter().enumerate() {
            check(item, item_schema, &format!("{path}[{index}]"))?;
        }
    }
    Ok(())
}

fn check_length(length: usize, schema: &Schema, path: &str) -> Result<(), String> {
    let length = length as u64;
    if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
        && length < min
    {
        return Err(format!("{path} needs at least {min} characters"));
    }
    if let Some(max) = schema.get("maxLength").and_then(Value::as_u64)
        && length > max
    {
        return Err(format!("{path} allows at most {max} characters"));
    }
    Ok(())
}

fn check_range(number: f64, schema: &Schema, path: &str) -> Result<(), String> {
    if let Some(min) = schema.get("minimum").and_then(Value::as_f64)
        && number < min
    {
        return Err(format!("{path} must be at least {min}"));
    }
    if let Some(max) = schema.get("maximum").and_then(Value::as_f64)
        && number > max
    {
        return Err(format!("{path} must be at most {max}"));
    }
    Ok(())
}

fn is_type(value: &Value, kind: &str) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => {
            value.is_i64()
                || value.is_u64()
                || value.as_f64().is_some_and(|number| number.fract() == 0.0)
        }
        _ => true,
    }
}

fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extracts_json_from_chatty_replies() {
        assert_eq!(extract(r#"{"a": 1}"#), Some(json!({ "a": 1 })));
        assert_eq!(
            extract("Sure!\n```json\n{\"a\": [1, 2]}\n```\nAnything else?"),
            Some(json!({ "a": [1, 2] }))
        );
        assert_eq!(
            extract("Here it is: {\"name\": \"Mira\"} hope it helps"),
            Some(json!({ "name": "Mira" }))
        );
        assert_eq!(extract("[1, 2, 3]"), Some(json!([1, 2, 3])));
        assert_eq!(extract("no json here"), None);
    }

    #[test]
    fn validates_objects() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "minLength": 1 },
                "age": { "type": "integer", "minimum": 0 },
                "tags": { "type": "array", "items": { "enum": ["a", "b"] } },
            },
            "required": ["name", "age"],
            "additionalProperties": false,
        });
        assert!(
            validate(
                &json!({ "name": "Mira", "age": 17, "tags": ["a"] }),
                &schema
            )
            .is_ok()
        );
        assert_eq!(
            validate(&json!({ "name": "Mira" }), &schema).unwrap_err(),
            "$.age is missing"
        );
        assert_eq!(
            validate(&json!({ "name": "Mira", "age": "17" }), &schema).unwrap_err(),
            "$.age must be integer, not a string"
        );
        assert_eq!(
            validate(&json!({ "name": "Mira", "age": 17, "extra": 1 }), &schema).unwrap_err(),
            "$.extra isn't an expected field"
        );
        assert!(
            validate(
                &json!({ "name": "Mira", "age": 17, "tags": ["c"] }),
                &schema
            )
            .unwrap_err()
            .starts_with("$.tags[0] must be one of")
        );
        assert!(validate(&json!({ "name": "Mira", "age": -1 }), &schema).is_err());
    }

    #[test]
    fn builds_an_example_with_the_schema_shape() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" },
                "mood": { "enum": ["happy", "sad"] },
                "tags": { "type": "array", "items": { "type": "string" } },
                "home": { "type": "object", "properties": { "city": { "type": "string" } } },
            },
        });
        assert_eq!(
            example(&schema),
            json!({ "name": "<name>", "age": 0, "mood": "happy", "tags": ["<tags item>"], "home": { "city": "<city>" } })
        );
    }

    #[test]
    fn handles_unions_and_whole_numbers() {
        let schema = json!({ "anyOf": [{ "type": "string" }, { "type": "null" }] });
        assert!(validate(&json!(null), &schema).is_ok());
        assert!(validate(&json!(3), &schema).is_err());
        assert!(validate(&json!(3.0), &json!({ "type": "integer" })).is_ok());
        assert!(validate(&json!([1]), &json!({ "type": "array", "minItems": 2 })).is_err());
    }
}
