//! In-memory fake backend.
//!
//! Lets the UI run on any machine (`CALCINE_BACKEND=mock`), including CI and
//! non-Snapdragon PCs, with realistic data shaped like real GenieX output.
//! Downloads are simulated with progress so the task drawer can be exercised.
//!
//! - `data`: the sample models, catalog, hub results and device
//! - `models`, `directory`, `runtime`, `hardware`, `server`: the service
//!   trait implementations

mod data;
mod directory;
mod hardware;
mod installer;
mod models;
mod runtime;
mod server;

pub use server::MockServer;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use calcine_core::jobs::JobManager;
use calcine_core::models::LocalModel;
use calcine_core::{BackendKind, Services};

#[derive(Debug)]
pub struct MockBackend {
    models: Mutex<Vec<LocalModel>>,
    /// Simulated download duration.
    pull_duration: Duration,
    /// Set with `config set chipset`; `None` means detected.
    chipset: Mutex<Option<String>>,
    /// "Installed" GenieX version, changed by simulated updates.
    version: Mutex<String>,
    /// Drives the simulated hardware load.
    started: Instant,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            models: Mutex::new(data::sample_models()),
            pull_duration: Duration::from_secs(6),
            chipset: Mutex::new(None),
            version: Mutex::new(data::runtime_info().cli_version),
            started: Instant::now(),
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
            directory: backend.clone(),
            runtime: backend.clone(),
            hardware: backend.clone(),
            installer: backend,
            server: Arc::new(MockServer::default()),
            jobs: JobManager::new(),
        }
    }

    fn models(&self) -> MutexGuard<'_, Vec<LocalModel>> {
        self.models.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn pinned_chipset(&self) -> MutexGuard<'_, Option<String>> {
        self.chipset.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn version(&self) -> MutexGuard<'_, String> {
        self.version.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn cli_version(&self) -> String {
        self.version().clone()
    }
}

#[cfg(test)]
mod tests {
    use calcine_core::jobs::JobState;
    use calcine_core::models::{ModelKey, ModelReference, ModelType, PullRequest};

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
        let request = PullRequest::download(ModelReference::parse("qualcomm/Qwen3-0.6B").unwrap());
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

    #[tokio::test]
    async fn set_type_clean_and_chipset() {
        let services = MockBackend::services();
        services
            .models
            .set_type("unsloth/Qwen3-0.6B-GGUF", ModelType::Vlm)
            .await
            .unwrap();
        let models = services.models.list().await.unwrap();
        assert!(
            models
                .iter()
                .any(|m| m.model_type == ModelType::Vlm && m.name.contains("0.6B"))
        );
        assert!(
            services
                .models
                .set_type("nope/nope", ModelType::Llm)
                .await
                .is_err()
        );

        services
            .runtime
            .set_chipset(Some("qualcomm-snapdragon-x-plus-8-core"))
            .await
            .unwrap();
        assert_eq!(
            services.runtime.chipset().await.unwrap().as_deref(),
            Some("qualcomm-snapdragon-x-plus-8-core")
        );
        services.runtime.set_chipset(None).await.unwrap();
        assert_eq!(
            services.runtime.chipset().await.unwrap().as_deref(),
            Some(data::CHIPSET)
        );

        services.models.clean().await.unwrap();
        assert_eq!(services.models.list().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn updates_geniex_and_reports_the_new_version() {
        use calcine_core::runtime::{InstallSource, ReleaseChannel};

        let services = MockBackend::default()
            .with_pull_duration(Duration::ZERO)
            .into_services();
        let check = services
            .installer
            .check(ReleaseChannel::Stable)
            .await
            .unwrap();
        assert!(check.update_available);
        let release = check.latest.unwrap();
        let id = services.start_runtime_install(InstallSource::Release { release });
        let mut events = services.jobs.subscribe();
        while services
            .jobs
            .get(id)
            .is_some_and(|job| job.state == JobState::Running)
        {
            let _ = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
        }
        assert_eq!(services.jobs.get(id).unwrap().state, JobState::Succeeded);
        assert_eq!(services.runtime.info().await.unwrap().cli_version, "v0.8.1");
        assert!(
            !services
                .installer
                .check(ReleaseChannel::Stable)
                .await
                .unwrap()
                .update_available
        );
    }

    #[tokio::test]
    async fn directory_finds_and_details_models() {
        let services = MockBackend::services();
        let found = services.directory.search("smol", 10).await.unwrap();
        assert!(found.iter().all(|m| m.name.to_lowercase().contains("smol")));
        let details = services
            .directory
            .details(&ModelReference::parse(&found[0].name).unwrap())
            .await
            .unwrap();
        assert_eq!(
            details.precisions.iter().filter(|p| p.recommended).count(),
            1
        );
        assert_ne!(services.directory.chipsets().await.unwrap().len(), 0);
    }
}

#[cfg(test)]
mod server_tests {
    use calcine_core::runtime::{InferenceServer, ServerState};

    use super::MockServer;

    #[tokio::test]
    async fn streams_a_canned_reply() {
        let server = MockServer::default();
        let url = server.ensure_running().await.unwrap();
        assert!(matches!(
            *server.state().borrow(),
            ServerState::Ready { .. }
        ));
        let body = reqwest::Client::new()
            .post(format!("{url}/v1/chat/completions"))
            .json(&serde_json::json!({
                "model": "mock",
                "stream": true,
                "messages": [{ "role": "user", "content": "hello" }],
            }))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(body.contains("reasoning_content"));
        assert!(body.contains("hello"));
        assert!(body.trim_end().ends_with("data:[DONE]"));
        server.stop().await.unwrap();
        assert_eq!(*server.state().borrow(), ServerState::Stopped);
    }
}
