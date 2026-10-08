//! Build the app state at startup and tear it down on exit.

mod gateway;
mod services;

use std::sync::Arc;

use calcine_core::Services;
use calcine_gateway::Gateway;
use tauri::{App, AppHandle, Manager};

pub use services::services_from_env;

/// Create and start the gateway, and forward state changes to the webview.
pub fn start(app: &App, services: &Services) -> tauri::Result<()> {
    let data_dir = app.path().app_data_dir()?;
    let gateway = Arc::new(gateway::create(services.clone(), &data_dir));
    app.manage(gateway.clone());
    crate::ipc::events::forward(app.handle(), &services.jobs, &gateway);
    tauri::async_runtime::spawn(async move {
        if let Err(err) = gateway.start().await {
            tracing::error!(%err, "gateway didn't start");
        }
    });
    Ok(())
}

/// Don't leave `geniex serve` (and a loaded model) running after quitting.
pub fn shutdown(handle: &AppHandle) {
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
