//! Calcine desktop shell.
//!
//! Wires the shared [`Services`] into Tauri. Commands stay thin: domain logic
//! lives in `calcine-core` and the backend crates so it can be tested (and
//! reused by the HTTP gateway) without Tauri.

mod bindings;
mod commands;
mod error;

use calcine_core::Services;
use calcine_geniex::{Geniex, GeniexConfig};
use calcine_mock::MockBackend;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Environment variable selecting the backend: `mock` or `geniex` (default).
const BACKEND_ENV: &str = "CALCINE_BACKEND";

/// Start the desktop app.
///
/// # Panics
///
/// If Tauri fails to start (for example, no WebView2 runtime).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let services = services_from_env();
    tracing::info!(backend = ?services.backend, "starting Calcine");

    let specta = bindings::builder();
    #[cfg(debug_assertions)]
    if let Err(err) = bindings::export(&specta) {
        tracing::warn!(%err, "couldn't export TypeScript bindings");
    }

    tauri::Builder::default()
        .manage(services)
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Calcine");
}

/// Regenerate `src/lib/bindings.ts` (used by `tests/bindings.rs`).
#[doc(hidden)]
pub fn export_bindings() -> Result<(), specta_typescript::Error> {
    bindings::export(&bindings::builder())
}

fn services_from_env() -> Services {
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
    let backend = Arc::new(Geniex::new(GeniexConfig::default()));
    Services {
        backend: calcine_core::BackendKind::Geniex,
        models: backend.clone(),
        runtime: backend,
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
