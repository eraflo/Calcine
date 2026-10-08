use std::fmt;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::reference::ModelHub;

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

/// A model found on a hub (Hugging Face search).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModel {
    /// Repository, ready to pull: `unsloth/Qwen3-0.6B-GGUF`.
    pub name: String,
    pub hub: ModelHub,
    /// `Unknown` when the hub doesn't say.
    pub model_type: ModelType,
    pub downloads: u64,
    pub likes: u64,
    /// ISO 8601 date of the last update.
    pub updated_at: Option<String>,
    /// Needs accepting a license on the hub website first.
    pub gated: bool,
}

/// What a remote model offers, to choose before downloading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModelDetails {
    pub name: String,
    pub hub: ModelHub,
    /// Guessed like GenieX does: a vision projector file means VLM.
    pub model_type: ModelType,
    /// The one GenieX picks when none is given comes first.
    pub precisions: Vec<RemotePrecision>,
    pub gated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemotePrecision {
    /// `Q4_0`, `Q8_0`, `F16`… as passed to `geniex pull name:precision`.
    pub name: String,
    /// Download size, including the vision projector for VLMs.
    pub size_bytes: u64,
    /// What GenieX downloads when no precision is given.
    pub recommended: bool,
}

/// A chipset Qualcomm AI Hub compiles models for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Chipset {
    /// Canonical id saved by `geniex config set chipset`, e.g.
    /// `qualcomm-snapdragon-x-elite`.
    pub id: String,
    /// Reference device, e.g. `Snapdragon X Elite CRD` (what GenieX reports
    /// when it detects the chipset itself).
    pub device: String,
    /// Marketing name, e.g. `Snapdragon X Elite`.
    pub marketing_name: Option<String>,
    /// Other names GenieX accepts (`sc8380xp`, …).
    pub aliases: Vec<String>,
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
