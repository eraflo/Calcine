//! System notifications when a download or import finishes while Calcine
//! isn't in front.

use calcine_core::jobs::{Job, JobKind, JobManager, JobState};
use tauri::{AppHandle, Manager};

use super::locale::{Locale, Strings};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::broadcast::error::RecvError;

pub fn notify_finished_jobs(app: &AppHandle, jobs: &JobManager) {
    let app = app.clone();
    let mut updates = jobs.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match updates.recv().await {
                Ok(job) if job.finished_at_ms.is_some() && !window_focused(&app) => {
                    let strings = app.state::<Locale>().get().strings();
                    if let Some((title, body)) = message(&job, &strings)
                        && let Err(err) =
                            app.notification().builder().title(title).body(body).show()
                    {
                        tracing::warn!(%err, "couldn't show a notification");
                    }
                }
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    });
}

fn window_focused(app: &AppHandle) -> bool {
    app.get_webview_window("main").is_some_and(|window| {
        window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false)
    })
}

/// Cancelled jobs were the user's own doing: no notification.
fn message(job: &Job, strings: &Strings) -> Option<(String, String)> {
    let (subject, messages, success) = match &job.kind {
        JobKind::Pull { model } => (model, &strings.download, strings.model_ready),
        JobKind::Import { model, .. } => (model, &strings.import, strings.model_ready),
        JobKind::InstallRuntime { version } => {
            (version, &strings.geniex_update, strings.version_installed)
        }
        JobKind::Benchmark { model } | JobKind::EnergyProfile { model } => {
            return match &job.state {
                JobState::Succeeded => Some((
                    strings.benchmark.done.to_owned(),
                    format!("{model}{}", strings.results_ready),
                )),
                JobState::Failed { message } => Some((
                    strings.benchmark.failed.to_owned(),
                    format!("{model}: {message}"),
                )),
                JobState::Running | JobState::Cancelled => None,
            };
        }
        // Calcine restarts into the new version: nothing to announce. The
        // benchmark tool downloads while its page is open.
        JobKind::UpdateApp { .. } | JobKind::InstallBench { .. } => return None,
    };
    match &job.state {
        JobState::Succeeded => Some((messages.done.to_owned(), format!("{subject} {success}"))),
        JobState::Failed { message } => {
            Some((messages.failed.to_owned(), format!("{subject}: {message}")))
        }
        JobState::Running | JobState::Cancelled => None,
    }
}
