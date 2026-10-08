//! Pick the backend (real GenieX or the mock) and assemble the services.

use std::path::PathBuf;
use std::sync::Arc;

use calcine_core::jobs::JobManager;
use calcine_core::{BackendKind, Services};
use calcine_geniex::update::RELEASE_ENDPOINT;
use calcine_geniex::{
    BundledInstaller, Geniex, GeniexConfig, GeniexInstaller, GeniexServer, InstallerConfig,
    ServeOptions,
};
use calcine_hub::{HubConfig, HubDirectory};
use calcine_hw::SystemProbe;
use calcine_mock::MockBackend;
use serde::Deserialize;

/// Environment variable selecting the backend: `mock` or `geniex` (default).
const BACKEND_ENV: &str = "CALCINE_BACKEND";

/// The GenieX version bundled in release installers (`runtime/geniex.json`).
const PINNED_GENIEX: &str = include_str!("../../../runtime/geniex.json");

/// Where Calcine keeps files the services need.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Downloaded GenieX installers, for rolling back.
    pub runtime_cache: PathBuf,
    /// The GenieX installer shipped in Calcine's installer, if any.
    pub bundled_geniex: Option<PathBuf>,
}

pub fn services_from_env(paths: &AppPaths) -> Services {
    match std::env::var(BACKEND_ENV).as_deref() {
        Ok("mock") => MockBackend::services(),
        Ok(other) if other != "geniex" => {
            tracing::warn!(
                value = other,
                "unknown {BACKEND_ENV}, falling back to geniex"
            );
            geniex_services(paths)
        }
        _ => geniex_services(paths),
    }
}

fn geniex_services(paths: &AppPaths) -> Services {
    let geniex = Geniex::new(GeniexConfig::default());
    let server = Arc::new(GeniexServer::new(geniex.clone(), ServeOptions::default()));
    let installer = GeniexInstaller::new(
        geniex.clone(),
        InstallerConfig {
            endpoint: RELEASE_ENDPOINT.to_owned(),
            cache_dir: paths.runtime_cache.clone(),
            bundled: paths.bundled_geniex.clone().map(|path| BundledInstaller {
                path,
                version: pinned_geniex_version(),
            }),
        },
    )
    .expect("the system TLS stack should load");
    let geniex = Arc::new(geniex);
    Services {
        backend: BackendKind::Geniex,
        models: geniex.clone(),
        catalog: geniex.clone(),
        directory: Arc::new(
            HubDirectory::new(HubConfig::default()).expect("the system TLS stack should load"),
        ),
        runtime: geniex,
        server,
        hardware: Arc::new(SystemProbe::new()),
        installer: Arc::new(installer),
        jobs: JobManager::new(),
    }
}

fn pinned_geniex_version() -> String {
    #[derive(Deserialize)]
    struct Pin {
        version: String,
    }
    serde_json::from_str::<Pin>(PINNED_GENIEX)
        .map(|pin| pin.version)
        .expect("runtime/geniex.json has a version")
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_the_pinned_version() {
        assert!(super::pinned_geniex_version().starts_with('v'));
    }
}
