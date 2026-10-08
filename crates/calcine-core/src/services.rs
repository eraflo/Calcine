use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::traits::{ModelStore, RuntimeManager};

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
    pub runtime: Arc<dyn RuntimeManager>,
}

impl fmt::Debug for Services {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Services")
            .field("backend", &self.backend)
            .finish_non_exhaustive()
    }
}
