//! Hugging Face Hub API payloads.

use calcine_core::models::{ModelHub, ModelType, RemoteModel, RemoteModelDetails};
use calcine_core::{Error, Result};
use serde::Deserialize;

use crate::quant::{self, RepoFile};

/// Fields requested from the search endpoint (keeps the reply small).
pub const SEARCH_FIELDS: &[&str] = &[
    "downloads",
    "likes",
    "lastModified",
    "gated",
    "pipeline_tag",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiModel {
    id: String,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    likes: u64,
    last_modified: Option<String>,
    /// `false`, `"auto"` or `"manual"`.
    #[serde(default)]
    gated: serde_json::Value,
    #[serde(default, rename = "pipeline_tag")]
    pipeline_tag: Option<String>,
    #[serde(default)]
    siblings: Vec<Sibling>,
}

#[derive(Debug, Deserialize)]
struct Sibling {
    rfilename: String,
    #[serde(default)]
    size: Option<u64>,
}

impl ApiModel {
    fn gated(&self) -> bool {
        !matches!(
            self.gated,
            serde_json::Value::Bool(false) | serde_json::Value::Null
        )
    }

    fn model_type(&self) -> ModelType {
        match self.pipeline_tag.as_deref() {
            Some("image-text-to-text" | "any-to-any" | "audio-text-to-text") => ModelType::Vlm,
            Some("text-generation") => ModelType::Llm,
            _ => ModelType::Unknown,
        }
    }
}

/// `GET /api/models?search=…`
pub fn parse_search(body: &str) -> Result<Vec<RemoteModel>> {
    let models: Vec<ApiModel> = serde_json::from_str(body).map_err(|err| parse_error(&err))?;
    Ok(models
        .into_iter()
        .map(|model| RemoteModel {
            hub: ModelHub::HuggingFace,
            model_type: model.model_type(),
            downloads: model.downloads,
            likes: model.likes,
            gated: model.gated(),
            updated_at: model.last_modified,
            name: model.id,
        })
        .collect())
}

/// `GET /api/models/{repo}?blobs=true`
pub fn parse_details(body: &str) -> Result<RemoteModelDetails> {
    let model: ApiModel = serde_json::from_str(body).map_err(|err| parse_error(&err))?;
    let files: Vec<RepoFile<'_>> = model
        .siblings
        .iter()
        .map(|file| RepoFile {
            path: &file.rfilename,
            size: file.size.unwrap_or_default(),
        })
        .collect();
    let found = quant::precisions(&files);
    let model_type = if found.has_projector {
        ModelType::Vlm
    } else {
        match model.model_type() {
            ModelType::Unknown if !found.precisions.is_empty() => ModelType::Llm,
            other => other,
        }
    };
    Ok(RemoteModelDetails {
        hub: ModelHub::HuggingFace,
        model_type,
        precisions: found.precisions,
        gated: model.gated(),
        name: model.id,
    })
}

fn parse_error(err: &serde_json::Error) -> Error {
    Error::Parse(format!("unexpected Hugging Face reply: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEARCH: &str = include_str!("../tests/fixtures/hf_search.json");
    const LLM: &str = include_str!("../tests/fixtures/hf_details_llm.json");
    const VLM: &str = include_str!("../tests/fixtures/hf_details_vlm.json");

    #[test]
    fn search_results_keep_hub_order() {
        let models = parse_search(SEARCH).unwrap();
        assert_eq!(models.len(), 5);
        assert_eq!(models[0].name, "unsloth/Qwen3.8-27B-GGUF");
        assert!(models[0].downloads > models[1].downloads);
        assert_eq!(models[1].model_type, ModelType::Llm);
        assert_eq!(models[2].model_type, ModelType::Vlm);
        assert!(models.iter().all(|model| !model.gated));
    }

    #[test]
    fn gguf_repo_lists_every_precision_with_q4_0_recommended() {
        let details = parse_details(LLM).unwrap();
        assert_eq!(details.name, "unsloth/Qwen3-0.6B-GGUF");
        assert_eq!(details.model_type, ModelType::Llm);
        let first = &details.precisions[0];
        assert_eq!(
            (first.name.as_str(), first.size_bytes),
            ("Q4_0", 382_156_480)
        );
        assert!(first.recommended);
        assert_eq!(
            details.precisions.iter().filter(|p| p.recommended).count(),
            1
        );
        for name in ["Q8_0", "BF16", "IQ4_XS", "Q2_K_L", "Q8_K_XL"] {
            assert!(
                details.precisions.iter().any(|p| p.name == name),
                "missing {name}"
            );
        }
    }

    #[test]
    fn projector_makes_a_vlm_and_counts_in_sizes() {
        let details = parse_details(VLM).unwrap();
        assert_eq!(details.model_type, ModelType::Vlm);
        let names: Vec<&str> = details.precisions.iter().map(|p| p.name.as_str()).collect();
        // No preferred tag among Q8_0 and F16? Q8_0 is preferred.
        assert_eq!(names, ["Q8_0", "F16"]);
        // Q8_0 weights + the F16 projector.
        assert_eq!(details.precisions[0].size_bytes, 175_054_528 + 190_031_616);
    }
}
