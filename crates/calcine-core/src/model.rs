use std::fmt;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Inference backend a cached model runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Runtime {
    /// Qualcomm AI Engine Direct: pre-compiled bundles, Hexagon NPU only.
    Qairt,
    /// llama.cpp: GGUF models on NPU, GPU or CPU.
    LlamaCpp,
    /// A runtime this version of Calcine doesn't know about yet.
    Unknown,
}

impl Runtime {
    /// Compute units this runtime can target.
    pub fn supported_compute(self) -> &'static [ComputeUnit] {
        match self {
            Self::Qairt => &[ComputeUnit::Npu],
            Self::LlamaCpp => &[
                ComputeUnit::Npu,
                ComputeUnit::Gpu,
                ComputeUnit::Cpu,
                ComputeUnit::Hybrid,
            ],
            Self::Unknown => &[],
        }
    }
}

/// Whether a model takes text only or also images and audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Llm,
    Vlm,
    /// A type this version of Calcine doesn't know about yet.
    Unknown,
}

/// Hardware a model can run on (the `--compute` flag of GenieX).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ComputeUnit {
    /// Hexagon NPU.
    Npu,
    /// Adreno GPU.
    Gpu,
    /// Oryon CPU.
    Cpu,
    /// Split between units.
    Hybrid,
}

/// A model in the local GenieX cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LocalModel {
    /// Cache name, e.g. `qualcomm/Qwen3-4B`. Also the API model id.
    pub name: String,
    /// Total size on disk in bytes.
    pub size_bytes: u64,
    pub runtime: Runtime,
    pub model_type: ModelType,
    /// Downloaded precisions (`Q4_0`, `W4A16`, …), recommended one first.
    pub precisions: Vec<String>,
}

/// A model from the Qualcomm AI Hub catalog (`geniex model list`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HubModel {
    /// Name ready to pull, e.g. `qualcomm/Qwen3-4B`.
    pub name: String,
    pub model_type: ModelType,
    /// Chipset slugs the model is compiled for (`x-elite`, `8gen3`, …). Only
    /// filled for the full catalog (`--all`).
    pub chipsets: Vec<String>,
}

/// The Qualcomm AI Hub catalog, optionally filtered for one chipset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HubCatalog {
    /// Chipset the list was filtered for (`None` for the full catalog).
    pub chipset: Option<String>,
    pub models: Vec<HubModel>,
}

/// A model reference, optionally pinned to one precision (`name:precision`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct ModelKey {
    pub name: String,
    pub precision: Option<String>,
}

impl fmt::Display for ModelKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.precision {
            Some(precision) => write!(f, "{}:{precision}", self.name),
            None => f.write_str(&self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_key_formats_like_the_cli() {
        let bare = ModelKey {
            name: "qualcomm/Qwen3-4B".into(),
            precision: None,
        };
        let pinned = ModelKey {
            precision: Some("Q4_0".into()),
            ..bare.clone()
        };
        assert_eq!(bare.to_string(), "qualcomm/Qwen3-4B");
        assert_eq!(pinned.to_string(), "qualcomm/Qwen3-4B:Q4_0");
    }

    #[test]
    fn serializes_like_the_cli() {
        assert_eq!(
            serde_json::to_string(&Runtime::LlamaCpp).unwrap(),
            "\"llama_cpp\""
        );
        assert_eq!(serde_json::to_string(&ModelType::Vlm).unwrap(), "\"vlm\"");
    }

    #[test]
    fn qairt_only_targets_the_npu() {
        assert_eq!(Runtime::Qairt.supported_compute(), &[ComputeUnit::Npu]);
    }
}
