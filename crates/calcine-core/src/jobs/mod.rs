//! Long-running work (downloads, updates, benchmarks) with progress and
//! cancellation, observable by any front door through [`JobManager::subscribe`].

mod manager;
mod types;

pub use manager::{JobCtx, JobManager};
pub use types::{Job, JobId, JobKind, JobProgress, JobState};
