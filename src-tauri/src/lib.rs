//! Calcine desktop shell.
//!
//! Wires the shared [`Services`] and the HTTP [`Gateway`] into Tauri. Commands
//! stay thin: domain logic lives in `calcine-core`, the backend crates and the
//! gateway, so it can be tested without Tauri.

mod bindings;
mod commands;
mod error;
mod events;
mod process;
mod tray;

use std::path::Path;
use std::sync::Arc;

use calcine_core::jobs::JobManager;
use calcine_core::{BackendKind, Services};
use calcine_gateway::{CALCINE_ORIGINS, Gateway, GatewayOptions, KeyStore};
use calcine_geniex::{Geniex, GeniexConfig, GeniexServer, ServeOptions};
use calcine_hw::SystemProbe;
use calcine_mock::MockBackend;
use tauri::{Manager, RunEvent, WindowEvent};
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
    process::kill_children_on_exit();

    let services = services_from_env();
    tracing::info!(backend = ?services.backend, "starting Calcine");

    let specta = bindings::builder();
    #[cfg(debug_assertions)]
    if let Err(err) = bindings::export(&specta) {
        tracing::warn!(%err, "couldn't export TypeScript bindings");
    }

    let app = tauri::Builder::default()
        // Must come first: a second launch focuses the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .manage(services.clone())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            let data_dir = app.path().app_data_dir()?;
            let gateway = Arc::new(create_gateway(services.clone(), &data_dir));
            app.manage(gateway.clone());
            events::forward(app.handle(), &services.jobs, &gateway);
            tauri::async_runtime::spawn(async move {
                if let Err(err) = gateway.start().await {
                    tracing::error!(%err, "gateway didn't start");
                }
            });
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
        .build(tauri::generate_context!())
        .expect("error while building Calcine");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // Don't leave `geniex serve` (and a loaded model) running after quitting.
            let services = handle.state::<Services>().inner().clone();
            let gateway = handle
                .try_state::<Arc<Gateway>>()
                .map(|state| state.inner().clone());
            tauri::async_runtime::block_on(async move {
                if let Some(gateway) = gateway {
                    gateway.stop().await;
                }
                let _ = services.server.stop().await;
            });
        }
    });
}

/// Regenerate `src/lib/bindings.ts` (used by `tests/bindings.rs`).
#[doc(hidden)]
pub fn export_bindings() -> Result<(), specta_typescript::Error> {
    bindings::export(&bindings::builder())
}

fn create_gateway(services: Services, data_dir: &Path) -> Gateway {
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
        },
    )
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
    let geniex = Geniex::new(GeniexConfig::default());
    let server = Arc::new(GeniexServer::new(geniex.clone(), ServeOptions::default()));
    let geniex = Arc::new(geniex);
    Services {
        backend: BackendKind::Geniex,
        models: geniex.clone(),
        catalog: geniex.clone(),
        runtime: geniex,
        server,
        hardware: Arc::new(SystemProbe),
        jobs: JobManager::new(),
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
