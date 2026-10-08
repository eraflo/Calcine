//! User-facing gateway settings, persisted as JSON.

use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;

pub const DEFAULT_PORT: u16 = 18181;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct GatewaySettings {
    /// Loopback port apps connect to. GenieX's default, so existing clients work.
    pub port: u16,
    /// Reject requests without a valid API key. Turning this off lets any
    /// local program use the models (inference only).
    pub require_api_key: bool,
    /// Extra browser origins allowed to call the API (e.g. a local web UI).
    pub allowed_origins: Vec<String>,
}

impl Default for GatewaySettings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            require_api_key: true,
            allowed_origins: Vec::new(),
        }
    }
}

impl GatewaySettings {
    /// Read settings, falling back to defaults when the file is missing or invalid.
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}
