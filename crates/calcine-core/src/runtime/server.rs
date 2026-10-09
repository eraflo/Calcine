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

/// Defaults `geniex serve` starts with. Changing them restarts the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerOptions {
    /// Unload the model after this many idle seconds (`--keepalive`).
    pub keepalive_secs: u32,
    /// Context window of llama.cpp models, in tokens (`--nctx`). AI Hub
    /// models have theirs compiled in.
    pub context_size: u32,
}

impl ServerOptions {
    pub const MIN_KEEPALIVE_SECS: u32 = 30;
    pub const MIN_CONTEXT_SIZE: u32 = 512;
    pub const MAX_CONTEXT_SIZE: u32 = 131_072;

    /// Why these options can't be used, if they can't.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.keepalive_secs < Self::MIN_KEEPALIVE_SECS {
            return Err(format!(
                "keep the model loaded for at least {} seconds",
                Self::MIN_KEEPALIVE_SECS
            ));
        }
        if !(Self::MIN_CONTEXT_SIZE..=Self::MAX_CONTEXT_SIZE).contains(&self.context_size) {
            return Err(format!(
                "choose a context window between {} and {} tokens",
                Self::MIN_CONTEXT_SIZE,
                Self::MAX_CONTEXT_SIZE
            ));
        }
        Ok(())
    }
}

impl Default for ServerOptions {
    /// GenieX's own defaults.
    fn default() -> Self {
        Self {
            keepalive_secs: 300,
            context_size: 4096,
        }
    }
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

    /// Options the server starts with.
    fn options(&self) -> ServerOptions;

    /// Change the options. A running server restarts with them.
    async fn set_options(&self, options: ServerOptions) -> Result<()>;

    /// Run a chat completion (not streamed) on the running server, bypassing
    /// the gateway. Used to measure it.
    async fn complete(&self, request: &serde_json::Value) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::ServerOptions;

    #[test]
    fn validates_ranges() {
        assert!(ServerOptions::default().validate().is_ok());
        let short = ServerOptions {
            keepalive_secs: 5,
            ..ServerOptions::default()
        };
        assert!(short.validate().is_err());
        let huge = ServerOptions {
            context_size: 1 << 20,
            ..ServerOptions::default()
        };
        assert!(huge.validate().is_err());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let options: ServerOptions = serde_json::from_str(r#"{"keepaliveSecs": 60}"#).unwrap();
        assert_eq!(options.context_size, 4096);
        assert_eq!(options.keepalive_secs, 60);
    }
}
