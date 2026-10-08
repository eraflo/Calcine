//! In-memory fake backend.
//!
//! Lets the UI run on any machine (`CALCINE_BACKEND=mock`), including CI and
//! non-Snapdragon PCs, with realistic data shaped like real GenieX output.

use std::sync::Arc;

use async_trait::async_trait;
use calcine_core::model::{LocalModel, ModelType, Runtime};
use calcine_core::runtime::RuntimeInfo;
use calcine_core::traits::{ModelStore, RuntimeManager};
use calcine_core::{BackendKind, Result, Services};

#[derive(Debug, Clone)]
pub struct MockBackend {
    models: Vec<LocalModel>,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            models: sample_models(),
        }
    }
}

impl MockBackend {
    /// Services wired to a fresh mock backend.
    pub fn services() -> Services {
        let backend = Arc::new(Self::default());
        Services {
            backend: BackendKind::Mock,
            models: backend.clone(),
            runtime: backend,
        }
    }
}

#[async_trait]
impl ModelStore for MockBackend {
    async fn list(&self) -> Result<Vec<LocalModel>> {
        Ok(self.models.clone())
    }
}

#[async_trait]
impl RuntimeManager for MockBackend {
    async fn info(&self) -> Result<RuntimeInfo> {
        Ok(RuntimeInfo {
            cli_version: "v0.8.0".into(),
            qairt_version: Some("2.45".into()),
            llama_cpp_hash: Some("9425611".into()),
            binary_path: "mock://geniex".into(),
        })
    }
}

fn sample_models() -> Vec<LocalModel> {
    vec![
        LocalModel {
            name: "qualcomm/Qwen3-4B".into(),
            size_bytes: 3_182_356_037,
            runtime: Runtime::Qairt,
            model_type: ModelType::Llm,
            precisions: vec!["W4A16".into()],
        },
        LocalModel {
            name: "unsloth/Qwen3-0.6B-GGUF".into(),
            size_bytes: 1_288_490_188,
            runtime: Runtime::LlamaCpp,
            model_type: ModelType::Llm,
            precisions: vec!["Q4_0".into(), "Q8_0".into()],
        },
        LocalModel {
            name: "google/gemma-4-E2B-it-qat-q4_0-gguf".into(),
            size_bytes: 4_294_967_296,
            runtime: Runtime::LlamaCpp,
            model_type: ModelType::Vlm,
            precisions: vec!["Q4_0".into()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_sample_data() {
        let services = MockBackend::services();
        assert_eq!(services.backend, BackendKind::Mock);
        assert_eq!(services.models.list().await.unwrap().len(), 3);
        assert_eq!(services.runtime.info().await.unwrap().cli_version, "v0.8.0");
    }
}
