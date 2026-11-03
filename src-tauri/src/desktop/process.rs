//! Make sure Calcine's child processes (`geniex serve`, downloads) never
//! outlive it, even if it crashes.

#[cfg(windows)]
use std::sync::OnceLock;

#[cfg(windows)]
use win32job::{ExtendedLimitInfo, Job};

/// The job must stay open for the app's lifetime.
#[cfg(windows)]
static JOB: OnceLock<Job> = OnceLock::new();

/// Put Calcine in a Windows Job Object that kills every process in it when
/// its last handle closes, which happens when Calcine exits for any reason.
/// Child processes join the job automatically.
#[cfg(windows)]
pub fn kill_children_on_exit() {
    let mut limits = ExtendedLimitInfo::new();
    limits.limit_kill_on_job_close();
    match Job::create_with_limit_info(&limits)
        .and_then(|job| job.assign_current_process().map(|()| job))
    {
        Ok(job) => {
            let _ = JOB.set(job);
        }
        Err(err) => tracing::warn!(%err, "couldn't tie child processes to Calcine's lifetime"),
    }
}

#[cfg(not(windows))]
pub fn kill_children_on_exit() {}

/// Let child processes outlive Calcine from now on: the update installer is
/// started right before Calcine exits, and must keep running to replace it.
/// Call only once `geniex serve` is stopped.
#[cfg(windows)]
pub fn let_children_outlive() {
    let Some(job) = JOB.get() else { return };
    if let Err(err) = job.set_extended_limit_info(ExtendedLimitInfo::new().clear_limits()) {
        tracing::warn!(%err, "the update installer may close with Calcine");
    }
}

#[cfg(not(windows))]
pub fn let_children_outlive() {}
