//! Model aliases: names apps ask for (`gpt-4o-mini`, `llama3`), answered by
//! a local model. Many apps only offer a fixed list of model names.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelAlias {
    /// The name apps ask for.
    pub alias: String,
    /// The local model that answers, e.g. `qualcomm/Qwen3-4B`.
    pub model: String,
}

/// The aliases, tidied, or why they can't be used.
pub fn validate(aliases: Vec<ModelAlias>) -> Result<Vec<ModelAlias>, String> {
    let aliases: Vec<ModelAlias> = aliases
        .into_iter()
        .map(|entry| ModelAlias {
            alias: without_latest(entry.alias.trim()).to_owned(),
            model: entry.model.trim().to_owned(),
        })
        .collect();
    for (index, entry) in aliases.iter().enumerate() {
        if entry.alias.is_empty() || entry.model.is_empty() {
            return Err("an alias needs a name and a model".into());
        }
        if entry.alias.eq_ignore_ascii_case(&entry.model) {
            return Err(format!("{} can't stand for itself", entry.alias));
        }
        if aliases[..index]
            .iter()
            .any(|earlier| earlier.alias.eq_ignore_ascii_case(&entry.alias))
        {
            return Err(format!("{} is listed twice", entry.alias));
        }
        if resolve(&aliases, &entry.model).is_some() {
            return Err(format!("{} is itself an alias", entry.model));
        }
    }
    Ok(aliases)
}

/// The model `name` stands for, when it's an alias. Case doesn't matter, nor
/// Ollama's `:latest`.
pub fn resolve<'a>(aliases: &'a [ModelAlias], name: &str) -> Option<&'a str> {
    let name = without_latest(name);
    aliases
        .iter()
        .find(|entry| entry.alias.eq_ignore_ascii_case(name))
        .map(|entry| entry.model.as_str())
}

/// Point `request` at the model its alias stands for; whether it changed.
pub fn apply(aliases: &[ModelAlias], request: &mut Value) -> bool {
    let Some(model) = request
        .get("model")
        .and_then(Value::as_str)
        .and_then(|name| resolve(aliases, name))
        .map(str::to_owned)
    else {
        return false;
    };
    request["model"] = Value::String(model);
    true
}

fn without_latest(name: &str) -> &str {
    name.strip_suffix(":latest").unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn alias(alias: &str, model: &str) -> ModelAlias {
        ModelAlias {
            alias: alias.into(),
            model: model.into(),
        }
    }

    #[test]
    fn resolves_whatever_the_case_and_tag() {
        let aliases = [alias("gpt-4o-mini", "qualcomm/Qwen3-4B")];
        assert_eq!(
            resolve(&aliases, "GPT-4o-mini:latest"),
            Some("qualcomm/Qwen3-4B")
        );
        assert_eq!(resolve(&aliases, "gpt-4o"), None);

        let mut request = json!({ "model": "gpt-4o-mini", "messages": [] });
        assert!(apply(&aliases, &mut request));
        assert_eq!(request["model"], "qualcomm/Qwen3-4B");
        assert!(!apply(&aliases, &mut request));
    }

    #[test]
    fn validates() {
        let tidy = validate(vec![alias(" llama3:latest ", " qualcomm/Qwen3-4B ")]).unwrap();
        assert_eq!(tidy, [alias("llama3", "qualcomm/Qwen3-4B")]);

        assert!(validate(vec![alias("", "m")]).is_err());
        assert!(validate(vec![alias("m", "M")]).is_err());
        assert!(validate(vec![alias("a", "m"), alias("A", "n")]).is_err());
        assert!(validate(vec![alias("a", "b"), alias("b", "m")]).is_err());
    }
}
