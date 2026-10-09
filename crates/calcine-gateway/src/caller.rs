//! Who sent a request, and what they may do.

use crate::keys::{ApiKeyInfo, KeyScope};

#[derive(Debug, Clone)]
pub enum Caller {
    /// Calcine's own UI (per-launch internal token).
    Calcine,
    /// An app with an API key.
    App(ApiKeyInfo),
    /// No key, accepted because keys aren't required (compatibility mode).
    Anonymous,
    /// No key, on the Ollama port: Ollama clients can't send one.
    OllamaClient,
}

impl Caller {
    pub fn can_infer(&self) -> bool {
        match self {
            Self::Calcine | Self::Anonymous | Self::OllamaClient => true,
            Self::App(key) => key.scopes.contains(&KeyScope::Inference),
        }
    }

    pub fn can_manage(&self) -> bool {
        match self {
            Self::Calcine => true,
            Self::App(key) => key.scopes.contains(&KeyScope::Manage),
            Self::Anonymous | Self::OllamaClient => false,
        }
    }

    /// Whether requests may make GenieX read local files or fetch URLs.
    pub fn allows_local_files(&self) -> bool {
        match self {
            Self::Calcine => true,
            Self::App(key) => key.allow_local_files,
            Self::Anonymous | Self::OllamaClient => false,
        }
    }

    /// Shown in the request log.
    pub fn label(&self) -> String {
        match self {
            Self::Calcine => "Calcine".into(),
            Self::App(key) => key.name.clone(),
            Self::Anonymous => "No key".into(),
            Self::OllamaClient => "Ollama app".into(),
        }
    }
}
