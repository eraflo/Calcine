//! Typed events pushed to the frontend.

use std::sync::Arc;

use calcine_core::jobs::{Job, JobManager};
use calcine_gateway::{Gateway, GatewayStatus, RequestEntry};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;
use tokio::sync::broadcast::{self, error::RecvError};

/// A job started, progressed or finished.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct JobUpdated(pub Job);

/// The gateway or the inference server changed state.
#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct GatewayUpdated(pub GatewayStatus);

/// An API request started or finished.
#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct RequestLogged(pub RequestEntry);

/// Forward core and gateway updates to the webview for the app's lifetime.
pub fn forward(app: &AppHandle, jobs: &JobManager, gateway: &Arc<Gateway>) {
    forward_broadcast(app.clone(), jobs.subscribe(), JobUpdated);
    forward_broadcast(app.clone(), gateway.subscribe_requests(), RequestLogged);

    let handle = app.clone();
    let mut status = gateway.subscribe_status();
    tauri::async_runtime::spawn(async move {
        while status.changed().await.is_ok() {
            let current = status.borrow_and_update().clone();
            if let Err(err) = GatewayUpdated(current).emit(&handle) {
                tracing::warn!(%err, "couldn't emit gateway status");
            }
        }
    });
}

fn forward_broadcast<T, E>(app: AppHandle, mut updates: broadcast::Receiver<T>, wrap: fn(T) -> E)
where
    T: Clone + Send + 'static,
    E: Event + Serialize + Clone,
{
    tauri::async_runtime::spawn(async move {
        loop {
            match updates.recv().await {
                Ok(update) => {
                    if let Err(err) = wrap(update).emit(&app) {
                        tracing::warn!(%err, "couldn't emit event");
                    }
                }
                // Updates are snapshots; skipping some under load is harmless.
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    });
}
