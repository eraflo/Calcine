//! In-memory fake backend.
//!
//! Lets the UI run on any machine (`CALCINE_BACKEND=mock`), including CI and
//! non-Snapdragon PCs, with realistic data shaped like real GenieX output.
//! Downloads are simulated with progress so the task drawer can be exercised.
//!
//! - `data`: the sample models, catalog and device
//! - `models`, `runtime`, `hardware`: the service trait implementations

mod data;
mod hardware;
mod models;
mod runtime;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use calcine_core::jobs::JobManager;
use calcine_core::models::LocalModel;
use calcine_core::{BackendKind, Services};

#[derive(Debug)]
pub struct MockBackend {
    models: Mutex<Vec<LocalModel>>,
    /// Simulated download duration.
    pull_duration: Duration,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            models: Mutex::new(data::sample_models()),
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

    fn models(&self) -> MutexGuard<'_, Vec<LocalModel>> {
        self.models.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use calcine_core::jobs::JobState;
    use calcine_core::models::{ModelKey, ModelReference, PullRequest};

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
        let request = PullRequest {
            reference: ModelReference::parse("qualcomm/Qwen3-0.6B").unwrap(),
            model_type: None,
        };
        let id = services.start_pull(request);
        let mut events = services.jobs.subscribe();
        while services
            .jobs
            .get(id)
            .is_some_and(|job| job.state == JobState::Running)
        {
            let _ = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
        }
        assert_eq!(services.jobs.get(id).unwrap().state, JobState::Succeeded);
        let names: Vec<String> = services
            .models
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert!(
            names.iter().any(|name| name == "qualcomm/Qwen3-0.6B"),
            "{names:?}"
        );
    }

    #[tokio::test]
    async fn removing_the_last_precision_removes_the_model() {
        let services = MockBackend::services();
        let key = ModelKey {
            name: "qualcomm/Qwen3-4B".into(),
            precision: Some("W4A16".into()),
        };
        services.models.remove(&[key]).await.unwrap();
        let models = services.models.list().await.unwrap();
        assert!(models.iter().all(|m| m.name != "qualcomm/Qwen3-4B"));
    }
}
