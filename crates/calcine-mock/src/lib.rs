//! In-memory fake backend.
//!
//! Lets the UI run on any machine (`CALCINE_BACKEND=mock`), including CI and
//! non-Snapdragon PCs, with realistic data shaped like real GenieX output.
//! Downloads are simulated with progress so the task drawer can be exercised.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use calcine_core::hardware::{Accelerator, DiskSpace, HardwareInfo, MemoryInfo, Processor};
use calcine_core::jobs::{JobCtx, JobManager, JobProgress};
use calcine_core::model::{
    ComputeUnit, HubCatalog, HubModel, LocalModel, ModelKey, ModelType, Runtime,
};
use calcine_core::reference::PullRequest;
use calcine_core::runtime::RuntimeInfo;
use calcine_core::traits::{HardwareProbe, ModelCatalog, ModelStore, RuntimeManager};
use calcine_core::{BackendKind, Error, Result, Services};

const CHIPSET: &str = "Snapdragon X Elite CRD";
const GIB: u64 = 1 << 30;

#[derive(Debug)]
pub struct MockBackend {
    models: Mutex<Vec<LocalModel>>,
    /// Simulated download duration.
    pull_duration: Duration,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            models: Mutex::new(sample_models()),
            pull_duration: Duration::from_secs(6),
        }
    }
}

impl MockBackend {
    /// Services wired to a fresh mock backend.
    pub fn services() -> Services {
        Self::default().into_services()
    }

    #[must_use]
    pub fn with_pull_duration(mut self, duration: Duration) -> Self {
        self.pull_duration = duration;
        self
    }

    pub fn into_services(self) -> Services {
        let backend = Arc::new(self);
        Services {
            backend: BackendKind::Mock,
            models: backend.clone(),
            catalog: backend.clone(),
            runtime: backend.clone(),
            hardware: backend,
            jobs: JobManager::new(),
        }
    }

    fn models(&self) -> std::sync::MutexGuard<'_, Vec<LocalModel>> {
        self.models.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl ModelStore for MockBackend {
    async fn list(&self) -> Result<Vec<LocalModel>> {
        Ok(self.models().clone())
    }

    async fn pull(&self, request: PullRequest, ctx: JobCtx) -> Result<()> {
        if request.reference.name.contains("missing") {
            return Err(Error::Command {
                command: format!("pull {}", request.reference.cli_arg()),
                message: "model not found".into(),
            });
        }
        let total = 760_000_000_u64;
        let steps = 30_u64;
        let tick = self.pull_duration / u32::try_from(steps).unwrap_or(1);
        let speed = total / self.pull_duration.as_secs().max(1);
        for step in 0..=steps {
            ctx.report(JobProgress {
                done_bytes: total * step / steps,
                total_bytes: Some(total),
                bytes_per_second: Some(speed),
            });
            tokio::select! {
                () = tokio::time::sleep(tick) => {}
                () = ctx.cancelled() => return Err(Error::Cancelled),
            }
        }

        let precision = request
            .reference
            .precision
            .clone()
            .unwrap_or_else(|| "W4A16".into());
        let mut models = self.models();
        models.retain(|model| model.name != request.reference.name);
        models.push(LocalModel {
            name: request.reference.name,
            size_bytes: total,
            runtime: Runtime::Qairt,
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
}

#[async_trait]
impl ModelCatalog for MockBackend {
    async fn aihub(&self, all_chipsets: bool) -> Result<HubCatalog> {
        let chipsets = |list: &[&str]| -> Vec<String> {
            if all_chipsets {
                list.iter().map(|c| (*c).to_owned()).collect()
            } else {
                Vec::new()
            }
        };
        let mut models = vec![
            hub(
                "qualcomm/Qwen3-0.6B",
                ModelType::Llm,
                chipsets(&["x-elite", "8gen3"]),
            ),
            hub(
                "qualcomm/Qwen3-4B",
                ModelType::Llm,
                chipsets(&["x-elite", "8gen3"]),
            ),
            hub(
                "qualcomm/Llama-v3.2-1B-Instruct",
                ModelType::Llm,
                chipsets(&["x-elite"]),
            ),
            hub(
                "qualcomm/Qwen3-VL-4B-Instruct",
                ModelType::Vlm,
                chipsets(&["x-elite"]),
            ),
        ];
        if all_chipsets {
            models.push(hub(
                "qualcomm/Phi-3.5-Mini-Instruct",
                ModelType::Llm,
                chipsets(&["8gen3"]),
            ));
        }
        Ok(HubCatalog {
            chipset: (!all_chipsets).then(|| CHIPSET.to_owned()),
            models,
        })
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

    async fn chipset(&self) -> Result<Option<String>> {
        Ok(Some(CHIPSET.into()))
    }

    fn models_dir(&self) -> Option<PathBuf> {
        Some(PathBuf::from("mock://models"))
    }
}

#[async_trait]
impl HardwareProbe for MockBackend {
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo> {
        Ok(HardwareInfo {
            cpu: Some(Processor {
                name: "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Oryon(TM) CPU".into(),
                cores: 12,
            }),
            memory: MemoryInfo {
                total_bytes: 32 * GIB,
                available_bytes: 18 * GIB,
            },
            accelerators: vec![
                Accelerator {
                    unit: ComputeUnit::Npu,
                    name: "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Hexagon(TM) NPU".into(),
                    driver_version: Some("30.0.219.1000".into()),
                },
                Accelerator {
                    unit: ComputeUnit::Gpu,
                    name: "Qualcomm(R) Adreno(TM) X1-85 GPU".into(),
                    driver_version: Some("31.0.133.1".into()),
                },
            ],
            models_disk: models_dir.map(|dir| DiskSpace {
                path: dir.display().to_string(),
                total_bytes: 1024 * GIB,
                available_bytes: 412 * GIB,
            }),
        })
    }
}

fn hub(name: &str, model_type: ModelType, chipsets: Vec<String>) -> HubModel {
    HubModel {
        name: name.into(),
        model_type,
        chipsets,
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
    use calcine_core::jobs::{JobKind, JobState};
    use calcine_core::reference::ModelReference;

    use super::*;

    #[tokio::test]
    async fn serves_sample_data() {
        let services = MockBackend::services();
        assert_eq!(services.backend, BackendKind::Mock);
        assert_eq!(services.models.list().await.unwrap().len(), 3);
        assert_eq!(services.runtime.info().await.unwrap().cli_version, "v0.8.0");
        assert_eq!(services.catalog.aihub(false).await.unwrap().models.len(), 4);
    }

    #[tokio::test]
    async fn pulled_models_appear_in_the_library() {
        let services = MockBackend::default()
            .with_pull_duration(Duration::ZERO)
            .into_services();
        let models = services.models.clone();
        let request = PullRequest {
            reference: ModelReference::parse("qualcomm/Qwen3-0.6B").unwrap(),
            model_type: None,
        };
        let id = services.jobs.spawn(
            JobKind::Pull {
                model: "qualcomm/Qwen3-0.6B".into(),
            },
            {
                let models = models.clone();
                move |ctx| async move { models.pull(request, ctx).await }
            },
        );
        let mut events = services.jobs.subscribe();
        while services
            .jobs
            .get(id)
            .is_some_and(|job| job.state == JobState::Running)
        {
            let _ = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
        }
        assert_eq!(services.jobs.get(id).unwrap().state, JobState::Succeeded);
        assert!(
            models
                .list()
                .await
                .unwrap()
                .iter()
                .any(|m| m.name == "qualcomm/Qwen3-0.6B")
        );
    }

    #[tokio::test]
    async fn removing_the_last_precision_removes_the_model() {
        let backend = MockBackend::default();
        let key = ModelKey {
            name: "qualcomm/Qwen3-4B".into(),
            precision: Some("W4A16".into()),
        };
        backend.remove(&[key]).await.unwrap();
        assert!(
            backend
                .list()
                .await
                .unwrap()
                .iter()
                .all(|m| m.name != "qualcomm/Qwen3-4B")
        );
    }
}
