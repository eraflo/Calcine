//! The HTTP gateway other apps call, with its keys and settings on disk.

use std::path::Path;
use std::sync::Arc;

use calcine_core::Services;
use calcine_gateway::{CALCINE_ORIGINS, Gateway, GatewayOptions, KeyStore};

pub fn create(services: Services, data_dir: &Path) -> Gateway {
    let keys = KeyStore::load(data_dir.join("api-keys.json")).unwrap_or_else(|err| {
        tracing::error!(%err, "couldn't read API keys; starting with none");
        KeyStore::in_memory()
    });
    let mut origins: Vec<String> = CALCINE_ORIGINS
        .iter()
        .map(|origin| (*origin).to_owned())
        .collect();
    if cfg!(debug_assertions) {
        // `tauri dev` serves the UI from Vite.
        origins.push("http://localhost:1420".to_owned());
    }
    Gateway::new(
        services,
        GatewayOptions {
            keys: Arc::new(keys),
            settings_path: Some(data_dir.join("gateway.json")),
            builtin_origins: origins,
            network_dir: Some(data_dir.join("network")),
        },
    )
}
