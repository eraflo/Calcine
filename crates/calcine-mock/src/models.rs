//! Simulated model cache and catalog.

use async_trait::async_trait;
use calcine_core::jobs::{JobCtx, JobProgress};
use calcine_core::models::{
    HubCatalog, LocalModel, ModelCatalog, ModelKey, ModelStore, ModelType, PullRequest, Runtime,
};
use calcine_core::{Error, Result};

use crate::{MockBackend, data};

/// Size of every simulated download.
const PULL_BYTES: u64 = 760_000_000;
const PULL_STEPS: u64 = 30;

#[async_trait]
impl ModelStore for MockBackend {
    async fn list(&self) -> Result<Vec<LocalModel>> {
        Ok(self.models().clone())
    }

    /// Fake a download with steady progress. Names containing `missing` fail,
    /// to exercise error states.
    async fn pull(&self, request: PullRequest, ctx: JobCtx) -> Result<()> {
        if request.reference.name.contains("missing") {
            return Err(Error::Command {
                command: format!("pull {}", request.reference.cli_arg()),
                message: "model not found".into(),
            });
        }
        let tick = self.pull_duration / u32::try_from(PULL_STEPS).unwrap_or(1);
        let speed = PULL_BYTES / self.pull_duration.as_secs().max(1);
        for step in 0..=PULL_STEPS {
            ctx.report(JobProgress {
                done_bytes: PULL_BYTES * step / PULL_STEPS,
                total_bytes: Some(PULL_BYTES),
                bytes_per_second: Some(speed),
            });
            tokio::select! {
                () = tokio::time::sleep(tick) => {}
                () = ctx.cancelled() => return Err(Error::Cancelled),
            }
        }

        // Hugging Face and local GGUF models run on llama.cpp.
        let (runtime, default_precision) = if request.reference.name.starts_with("qualcomm/") {
            (Runtime::Qairt, "W4A16")
        } else {
            (Runtime::LlamaCpp, "Q4_0")
        };
        let precision = request
            .reference
            .precision
            .clone()
            .unwrap_or_else(|| default_precision.into());
        let mut models = self.models();
        models.retain(|model| model.name != request.reference.name);
        models.push(LocalModel {
            name: request.reference.name,
            size_bytes: PULL_BYTES,
            runtime,
            model_type: request.model_type.unwrap_or(ModelType::Llm),
            precisions: vec![precision],
        });
        Ok(())
    }

    async fn remove(&self, keys: &[ModelKey]) -> Result<()> {
        let mut models = self.models();
        for key in keys {
            match &key.precision {
                None => models.retain(|model| model.name != key.name),
                Some(precision) => {
                    for model in models.iter_mut().filter(|model| model.name == key.name) {
                        model.precisions.retain(|p| p != precision);
                    }
                    models.retain(|model| !model.precisions.is_empty());
                }
            }
        }
        Ok(())
    }

    async fn clean(&self) -> Result<()> {
        self.models().clear();
        Ok(())
    }

    async fn set_type(&self, name: &str, model_type: ModelType) -> Result<()> {
        let mut models = self.models();
        let model = models
            .iter_mut()
            .find(|model| model.name == name)
            .ok_or_else(|| Error::InvalidInput(format!("{name} isn't downloaded")))?;
        model.model_type = model_type;
        Ok(())
    }
}

#[async_trait]
impl ModelCatalog for MockBackend {
    async fn aihub(&self, all_chipsets: bool) -> Result<HubCatalog> {
        Ok(data::catalog(all_chipsets))
    }
}
