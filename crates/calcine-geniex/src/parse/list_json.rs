//! `geniex list --format json` (stable schema documented by GenieX).

use calcine_core::model::LocalModel;
use calcine_core::{Error, Result};
use serde::Deserialize;

/// Unknown runtime or type strings map to `Unknown` instead of failing, so a
/// newer GenieX doesn't break the whole list.
#[derive(Debug, Deserialize)]
struct ListedModel {
    name: String,
    size: u64,
    runtime: String,
    #[serde(rename = "type")]
    model_type: String,
    #[serde(default)]
    precisions: Vec<String>,
}

pub fn parse(output: &str) -> Result<Vec<LocalModel>> {
    let listed: Vec<ListedModel> =
        serde_json::from_str(output).map_err(|err| Error::Parse(format!("model list: {err}")))?;
    Ok(listed
        .into_iter()
        .map(|model| LocalModel {
            name: model.name,
            size_bytes: model.size,
            runtime: super::runtime(&model.runtime),
            model_type: super::model_type(&model.model_type),
            precisions: model.precisions,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::fixture;
    use calcine_core::model::{ModelType, Runtime};

    #[test]
    fn parses_real_output() {
        let models = parse(&fixture("list.json")).unwrap();
        assert_eq!(
            models,
            vec![LocalModel {
                name: "qualcomm/Qwen3-4B".into(),
                size_bytes: 3_182_356_037,
                runtime: Runtime::Qairt,
                model_type: ModelType::Llm,
                precisions: vec!["W4A16".into()],
            }]
        );
    }

    #[test]
    fn parses_mixed_runtimes_and_types() {
        let models = parse(&fixture("list_mixed.json")).unwrap();
        assert_eq!(models.len(), 3);
        assert_eq!(models[1].runtime, Runtime::LlamaCpp);
        assert_eq!(models[1].precisions, ["Q4_0", "Q8_0"]);
        assert_eq!(models[2].model_type, ModelType::Vlm);
    }

    #[test]
    fn unknown_runtime_and_type_do_not_fail() {
        let output =
            r#"[{"name":"x/y","size":1,"runtime":"onnx","type":"embedding","precisions":[]}]"#;
        let models = parse(output).unwrap();
        assert_eq!(models[0].runtime, Runtime::Unknown);
        assert_eq!(models[0].model_type, ModelType::Unknown);
    }

    #[test]
    fn empty_cache_is_an_empty_list() {
        assert_eq!(parse("[]").unwrap(), []);
    }

    #[test]
    fn garbage_is_a_parse_error() {
        assert!(matches!(parse("Models cached in …"), Err(Error::Parse(_))));
    }
}
