//! Typed events pushed to the frontend.

use calcine_core::jobs::{Job, JobManager};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;
use tokio::sync::broadcast::error::RecvError;

/// A job started, progressed or finished.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct JobUpdated(pub Job);

/// Forward job updates from the core to the webview for the app's lifetime.
pub fn forward_jobs(app: AppHandle, jobs: &JobManager) {
    let mut updates = jobs.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match updates.recv().await {
                Ok(job) => {
                    if let Err(err) = JobUpdated(job).emit(&app) {
                        tracing::warn!(%err, "couldn't emit job update");
                    }
                }
                // Progress events are throttled snapshots; skipping some is harmless.
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    });
}
