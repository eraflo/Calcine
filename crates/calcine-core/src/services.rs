use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Result;
use crate::hardware::HardwareProbe;
use crate::jobs::{JobId, JobKind, JobManager, JobState};
use crate::models::{ModelCatalog, ModelDirectory, ModelKey, ModelStore, PullRequest};
use crate::runtime::{InferenceServer, RuntimeManager};

/// Which implementation backs the services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// The real GenieX CLI.
    Geniex,
    /// In-memory fake data (`CALCINE_BACKEND=mock`).
    Mock,
}

/// The single set of services shared by Tauri commands and the HTTP gateway.
#[derive(Clone)]
pub struct Services {
    pub backend: BackendKind,
    pub models: Arc<dyn ModelStore>,
    pub catalog: Arc<dyn ModelCatalog>,
    /// Hugging Face and AI Hub over HTTP.
    pub directory: Arc<dyn ModelDirectory>,
    pub runtime: Arc<dyn RuntimeManager>,
    /// `geniex serve`, started on demand by the gateway.
    pub server: Arc<dyn InferenceServer>,
    pub hardware: Arc<dyn HardwareProbe>,
    pub jobs: JobManager,
}

impl Services {
    /// Start downloading a model, or return the job already downloading it.
    pub fn start_pull(&self, request: PullRequest) -> JobId {
        let model = request.reference.cli_arg();
        let kind = if request.is_import() {
            JobKind::Import {
                model,
                path: request.local_path.clone().unwrap_or_default(),
            }
        } else {
            JobKind::Pull { model }
        };
        if let Some(job) = self
            .jobs
            .list()
            .into_iter()
            .find(|job| job.kind == kind && job.state == JobState::Running)
        {
            return job.id;
        }
        let store = self.models.clone();
        self.jobs.spawn(
            kind,
            move |ctx| async move { store.pull(request, ctx).await },
        )
    }
}

impl Services {
    /// Delete models or precisions. Stops `geniex serve` first: Windows
    /// can't delete the files of a loaded model.
    pub async fn remove_models(&self, keys: &[ModelKey]) -> Result<()> {
        self.server.stop().await?;
        self.models.remove(keys).await
    }

    /// Delete every cached model, stopping `geniex serve` first.
    pub async fn clean_models(&self) -> Result<()> {
        self.server.stop().await?;
        self.models.clean().await
    }
}

impl fmt::Debug for Services {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Services")
            .field("backend", &self.backend)
            .finish_non_exhaustive()
    }
}
