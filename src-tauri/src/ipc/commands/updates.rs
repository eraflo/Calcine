//! Updating Calcine itself (signed updates from GitHub releases).

use calcine_core::Services;
use calcine_core::jobs::{JobId, JobKind, JobPhase, JobProgress, JobState};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::ipc::error::{ApiError, ApiResult};

/// Which Calcine releases to follow.
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AppChannel {
    /// Releases from `main`.
    Stable,
    /// Pre-releases from `dev` (or the latest stable when it's newer).
    Beta,
}

impl AppChannel {
    fn endpoint(self) -> &'static str {
        match self {
            Self::Stable => {
                "https://github.com/eraflo/calcine/releases/latest/download/latest.json"
            }
            Self::Beta => {
                "https://github.com/eraflo/calcine/releases/download/beta-channel/latest-beta.json"
            }
        }
    }
}

/// A newer Calcine, ready to install.
#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdate {
    pub version: String,
    pub current_version: String,
    /// Release notes (Markdown).
    pub notes: Option<String>,
    /// RFC 3339 date.
    pub date: Option<String>,
}

async fn find_update(app: &AppHandle, channel: AppChannel) -> ApiResult<Option<Update>> {
    let endpoint = channel
        .endpoint()
        .parse()
        .map_err(|err| ApiError::invalid_input(format!("bad update URL: {err}")))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .and_then(tauri_plugin_updater::UpdaterBuilder::build)
        .map_err(network)?;
    updater.check().await.map_err(|err| match err {
        tauri_plugin_updater::Error::ReleaseNotFound => ApiError {
            kind: calcine_core::ErrorKind::Network,
            message: "no Calcine release is published on this channel yet".into(),
        },
        other => network(other),
    })
}

fn network(err: impl std::fmt::Display) -> ApiError {
    ApiError {
        kind: calcine_core::ErrorKind::Network,
        message: format!("couldn't check for Calcine updates: {err}"),
    }
}

/// The newest Calcine on `channel`, or `None` when up to date.
#[tauri::command]
#[specta::specta]
pub async fn check_app_update(app: AppHandle, channel: AppChannel) -> ApiResult<Option<AppUpdate>> {
    Ok(find_update(&app, channel).await?.map(|update| AppUpdate {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
        date: update.date.map(|date| date.to_string()),
    }))
}

/// Download the update with progress (a job in the task drawer), verify its
/// signature, then run the installer, which closes Calcine and reopens it.
#[tauri::command]
#[specta::specta]
pub async fn install_app_update(
    app: AppHandle,
    services: State<'_, Services>,
    channel: AppChannel,
) -> ApiResult<JobId> {
    if let Some(job) =
        services.jobs.list().into_iter().find(|job| {
            matches!(job.kind, JobKind::UpdateApp { .. }) && job.state == JobState::Running
        })
    {
        return Ok(job.id);
    }
    let update = find_update(&app, channel)
        .await?
        .ok_or_else(|| ApiError::invalid_input("Calcine is up to date".into()))?;
    let kind = JobKind::UpdateApp {
        version: update.version.clone(),
    };
    let server = services.server.clone();
    Ok(services.jobs.spawn(kind, move |ctx| async move {
        let mut done = 0_u64;
        let progress_ctx = ctx.clone();
        let bytes = update
            .download(
                move |chunk, total| {
                    done += chunk as u64;
                    progress_ctx.report(JobProgress {
                        done_bytes: done,
                        total_bytes: total,
                        bytes_per_second: None,
                        phase: Some(JobPhase::Downloading),
                        step: None,
                    });
                },
                || {},
            )
            .await
            .map_err(|err| {
                calcine_core::Error::Network(format!("update download failed: {err}"))
            })?;
        ctx.report(JobProgress {
            phase: Some(JobPhase::Installing),
            ..JobProgress::default()
        });
        // Don't leave `geniex serve` behind: the installer closes Calcine.
        let _ = server.stop().await;
        update
            .install(bytes)
            .map_err(|err| calcine_core::Error::Command {
                command: "Calcine installer".into(),
                message: err.to_string(),
            })?;
        app.restart();
    }))
}
