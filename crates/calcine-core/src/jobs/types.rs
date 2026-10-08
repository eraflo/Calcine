//! Job data shared with the UI.

use serde::{Deserialize, Serialize};
use specta::Type;

pub type JobId = u32;

/// What a job does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobKind {
    /// Downloading a model (`geniex pull`).
    Pull { model: String },
    /// Copying a model from this PC into the cache (`geniex pull --model-hub localfs`).
    Import { model: String, path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobState {
    Running,
    Succeeded,
    Cancelled,
    Failed { message: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub done_bytes: u64,
    pub total_bytes: Option<u64>,
    pub bytes_per_second: Option<u64>,
}

/// A snapshot of one job, as sent to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub state: JobState,
    pub progress: Option<JobProgress>,
    pub started_at_ms: u64,
    pub finished_at_ms: Option<u64>,
}
