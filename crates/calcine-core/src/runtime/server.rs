//! The inference server (`geniex serve`) behind Calcine's gateway.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::watch;

use crate::Result;

/// Lifecycle of the inference server process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(
    tag = "state",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ServerState {
    Stopped,
    Starting,
    /// Accepting requests at `url` (loopback, internal port).
    Ready {
        url: String,
        started_at_ms: u64,
    },
    /// The process exited or never became ready.
    Failed {
        message: String,
    },
}

/// Runs the OpenAI-compatible inference server on demand.
#[async_trait]
pub trait InferenceServer: Send + Sync {
    /// Start the server if needed and return its base URL (no `/v1`).
    /// Concurrent callers share one start-up.
    async fn ensure_running(&self) -> Result<String>;

    /// Stop the server. Requests in flight are cut off.
    async fn stop(&self) -> Result<()>;

    /// Current state, updated on every transition.
    fn state(&self) -> watch::Receiver<ServerState>;

    /// Recent output lines of the server process, oldest first.
    fn logs(&self) -> Vec<String>;
}
