//! Calcine desktop shell.
//!
//! Wires the shared [`Services`] into Tauri. Commands stay thin: domain logic
//! lives in `calcine-core` and the backend crates so it can be tested (and
//! reused by the HTTP gateway) without Tauri.

mod bindings;
mod commands;
mod error;
mod events;
mod tray;

use std::sync::Arc;

use calcine_core::jobs::JobManager;
use calcine_core::{BackendKind, Services};
use calcine_geniex::{Geniex, GeniexConfig};
use calcine_hw::SystemProbe;
use calcine_mock::MockBackend;
use tauri::{Manager, WindowEvent};
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
        // Must come first: a second launch focuses the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .manage(services)
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            events::forward_jobs(app.handle().clone(), &app.state::<Services>().jobs);
            tray::install(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window keeps Calcine in the tray; quit from the tray menu.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
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
    let geniex = Arc::new(Geniex::new(GeniexConfig::default()));
    Services {
        backend: BackendKind::Geniex,
        models: geniex.clone(),
        catalog: geniex.clone(),
        runtime: geniex,
        hardware: Arc::new(SystemProbe),
        jobs: JobManager::new(),
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
