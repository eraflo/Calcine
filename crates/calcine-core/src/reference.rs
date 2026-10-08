//! What the user types or pastes to download a model.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::ModelType;
use crate::{Error, Result};

/// Where GenieX downloads a model from (`--model-hub`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ModelHub {
    /// Let GenieX decide from the name.
    Auto,
    /// Qualcomm AI Hub (pre-compiled NPU bundles).
    AiHub,
    HuggingFace,
    ModelScope,
    DockerHub,
}

impl ModelHub {
    /// Value for `--model-hub`, or `None` for auto-detection.
    pub fn cli_value(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::AiHub => Some("aihub"),
            Self::HuggingFace => Some("hf"),
            Self::ModelScope => Some("modelscope"),
            Self::DockerHub => Some("docker"),
        }
    }
}

/// A model to download: `name[:precision]` plus the hub it comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ModelReference {
    /// Repository name, e.g. `unsloth/Qwen3-0.6B-GGUF` or `ai/gemma3`.
    pub name: String,
    /// GGUF quantization (`Q4_0`) or Docker tag. `None` lets GenieX pick the
    /// recommended one.
    pub precision: Option<String>,
    pub hub: ModelHub,
}

impl ModelReference {
    /// Understand a pasted name or URL:
    ///
    /// - `org/repo`, `org/repo:Q4_0`
    /// - `https://huggingface.co/org/repo[/…]` (also `hf.co`)
    /// - `https://modelscope.cn/models/org/repo[/…]`
    /// - `docker.io/ai/gemma3[:tag]`, `https://hub.docker.com/r/ai/gemma3`
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            return Err(invalid("enter a model name or link"));
        }

        let without_scheme = input
            .strip_prefix("https://")
            .or_else(|| input.strip_prefix("http://"))
            .unwrap_or(input);
        let without_query = without_scheme.split(['?', '#']).next().unwrap_or_default();
        let (host, path) = without_query.split_once('/').unwrap_or(("", without_query));
        let host = host.trim_start_matches("www.").to_ascii_lowercase();

        let (hub, path) = match host.as_str() {
            "huggingface.co" | "hf.co" => (ModelHub::HuggingFace, path),
            "modelscope.cn" | "modelscope.ai" => (
                ModelHub::ModelScope,
                path.strip_prefix("models/").unwrap_or(path),
            ),
            "docker.io" | "registry-1.docker.io" => (ModelHub::DockerHub, path),
            "hub.docker.com" => (ModelHub::DockerHub, path.strip_prefix("r/").unwrap_or(path)),
            _ => (ModelHub::Auto, without_query),
        };

        // Keep `owner/repo[:precision]`; drop trailing paths such as `/tree/main`.
        let mut segments = path.split('/').filter(|segment| !segment.is_empty());
        let (Some(owner), Some(repo)) = (segments.next(), segments.next()) else {
            return Err(invalid("use the form owner/model, e.g. qualcomm/Qwen3-4B"));
        };
        let (repo, precision) = match repo.split_once(':') {
            Some((repo, precision)) => (repo, Some(precision.trim().to_owned())),
            None => (repo, None),
        };
        if owner.contains(char::is_whitespace) || repo.is_empty() || repo.contains(' ') {
            return Err(invalid("model names can't contain spaces"));
        }

        Ok(Self {
            name: format!("{owner}/{repo}"),
            precision: precision.filter(|p| !p.is_empty()),
            hub,
        })
    }

    /// The positional argument of `geniex pull`.
    pub fn cli_arg(&self) -> String {
        match &self.precision {
            Some(precision) => format!("{}:{precision}", self.name),
            None => self.name.clone(),
        }
    }
}

/// A download request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub reference: ModelReference,
    /// Override GenieX's LLM/VLM detection.
    pub model_type: Option<ModelType>,
}

fn invalid(message: &str) -> Error {
    Error::InvalidInput(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> ModelReference {
        ModelReference::parse(input).unwrap_or_else(|err| panic!("{input}: {err}"))
    }

    #[test]
    fn plain_names_use_auto_detection() {
        let reference = parse("qualcomm/Qwen3-0.6B");
        assert_eq!(reference.name, "qualcomm/Qwen3-0.6B");
        assert_eq!(reference.precision, None);
        assert_eq!(reference.hub, ModelHub::Auto);
    }

    #[test]
    fn precision_suffix_is_split_off() {
        let reference = parse(" unsloth/Qwen3-0.6B-GGUF:Q4_0 ");
        assert_eq!(reference.name, "unsloth/Qwen3-0.6B-GGUF");
        assert_eq!(reference.precision.as_deref(), Some("Q4_0"));
        assert_eq!(reference.cli_arg(), "unsloth/Qwen3-0.6B-GGUF:Q4_0");
    }

    #[test]
    fn hugging_face_links_point_at_the_repo() {
        for link in [
            "https://huggingface.co/unsloth/Qwen3-0.6B-GGUF",
            "huggingface.co/unsloth/Qwen3-0.6B-GGUF/tree/main",
            "https://hf.co/unsloth/Qwen3-0.6B-GGUF?show_file_info=x",
        ] {
            let reference = parse(link);
            assert_eq!(reference.name, "unsloth/Qwen3-0.6B-GGUF", "{link}");
            assert_eq!(reference.hub, ModelHub::HuggingFace, "{link}");
        }
    }

    #[test]
    fn modelscope_links_drop_the_models_prefix() {
        let reference = parse("https://modelscope.cn/models/Qwen/Qwen3-0.6B-GGUF");
        assert_eq!(reference.name, "Qwen/Qwen3-0.6B-GGUF");
        assert_eq!(reference.hub, ModelHub::ModelScope);
    }

    #[test]
    fn docker_references_keep_the_tag_as_precision() {
        let reference = parse("docker.io/ai/gemma3:latest");
        assert_eq!(reference.name, "ai/gemma3");
        assert_eq!(reference.precision.as_deref(), Some("latest"));
        assert_eq!(reference.hub, ModelHub::DockerHub);
        assert_eq!(
            parse("https://hub.docker.com/r/ai/gemma3").name,
            "ai/gemma3"
        );
    }

    #[test]
    fn rejects_incomplete_input() {
        for input in [
            "",
            "   ",
            "qwen",
            "https://huggingface.co/unsloth",
            "my org/model",
        ] {
            assert!(
                ModelReference::parse(input).is_err(),
                "{input:?} should be rejected"
            );
        }
    }

    #[test]
    fn hub_flags_match_the_cli() {
        assert_eq!(ModelHub::Auto.cli_value(), None);
        assert_eq!(ModelHub::HuggingFace.cli_value(), Some("hf"));
        assert_eq!(ModelHub::DockerHub.cli_value(), Some("docker"));
    }
}
