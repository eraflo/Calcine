//! Make sure Calcine's child processes (`geniex serve`, downloads) never
//! outlive it, even if it crashes.

/// Put Calcine in a Windows Job Object that kills every process in it when
/// its last handle closes, which happens when Calcine exits for any reason.
/// Child processes join the job automatically.
#[cfg(windows)]
pub fn kill_children_on_exit() {
    use std::sync::OnceLock;

    use win32job::{ExtendedLimitInfo, Job};

    // The job must stay open for the app's lifetime.
    static JOB: OnceLock<Job> = OnceLock::new();

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
