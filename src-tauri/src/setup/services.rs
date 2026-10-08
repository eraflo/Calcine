//! Pick the backend (real GenieX or the mock) and assemble the services.

use std::sync::Arc;

use calcine_core::jobs::JobManager;
use calcine_core::{BackendKind, Services};
use calcine_geniex::{Geniex, GeniexConfig, GeniexServer, ServeOptions};
use calcine_hub::{HubConfig, HubDirectory};
use calcine_hw::SystemProbe;
use calcine_mock::MockBackend;

/// Environment variable selecting the backend: `mock` or `geniex` (default).
const BACKEND_ENV: &str = "CALCINE_BACKEND";

pub fn services_from_env() -> Services {
    match std::env::var(BACKEND_ENV).as_deref() {
        Ok("mock") => MockBackend::services(),
        Ok(other) if other != "geniex" => {
            tracing::warn!(
                value = other,
                "unknown {BACKEND_ENV}, falling back to geniex"
            );
            geniex_services()
        }
        _ => geniex_services(),
    }
}

fn geniex_services() -> Services {
    let geniex = Geniex::new(GeniexConfig::default());
    let server = Arc::new(GeniexServer::new(geniex.clone(), ServeOptions::default()));
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
        jobs: JobManager::new(),
    }
}
