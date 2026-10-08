use calcine_core::Services;
use calcine_core::jobs::{Job, JobId};
use tauri::State;

/// Every known job, newest first.
#[tauri::command]
#[specta::specta]
pub fn list_jobs(services: State<'_, Services>) -> Vec<Job> {
    services.jobs.list()
}

/// Ask a running job to stop. Returns `false` if it already finished.
#[tauri::command]
#[specta::specta]
pub fn cancel_job(services: State<'_, Services>, id: JobId) -> bool {
    services.jobs.cancel(id)
}

/// Remove a finished job from the list.
#[tauri::command]
#[specta::specta]
pub fn dismiss_job(services: State<'_, Services>, id: JobId) -> bool {
    services.jobs.dismiss(id)
}
