//! Runs jobs on Tokio and tracks their state, progress and cancellation.

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use super::types::{Job, JobId, JobKind, JobProgress, JobState};
use crate::{Error, Result};

/// Minimum delay between two progress events for the same job.
const PROGRESS_THROTTLE: Duration = Duration::from_millis(200);

#[derive(Debug)]
struct Entry {
    job: Job,
    cancel: CancellationToken,
    last_progress_event: Option<Instant>,
}

#[derive(Debug)]
struct Inner {
    jobs: Mutex<HashMap<JobId, Entry>>,
    next_id: AtomicU32,
    events: broadcast::Sender<Job>,
}

/// Runs and tracks jobs. Cheap to clone.
#[derive(Debug, Clone)]
pub struct JobManager {
    inner: Arc<Inner>,
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobManager {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(Inner {
                jobs: Mutex::new(HashMap::new()),
                next_id: AtomicU32::new(1),
                events,
            }),
        }
    }

    /// Every job update (start, throttled progress, completion).
    pub fn subscribe(&self) -> broadcast::Receiver<Job> {
        self.inner.events.subscribe()
    }

    /// All known jobs, newest first.
    pub fn list(&self) -> Vec<Job> {
        let mut jobs: Vec<Job> = self
            .lock()
            .values()
            .map(|entry| entry.job.clone())
            .collect();
        jobs.sort_by_key(|job| std::cmp::Reverse(job.id));
        jobs
    }

    pub fn get(&self, id: JobId) -> Option<Job> {
        self.lock().get(&id).map(|entry| entry.job.clone())
    }

    /// Start `run` on the Tokio runtime. Returning [`Error::Cancelled`] marks
    /// the job cancelled; any other error marks it failed.
    pub fn spawn<F, Fut>(&self, kind: JobKind, run: F) -> JobId
    where
        F: FnOnce(JobCtx) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let cancel = CancellationToken::new();
        let job = Job {
            id,
            kind,
            state: JobState::Running,
            progress: None,
            started_at_ms: now_ms(),
            finished_at_ms: None,
        };
        self.lock().insert(
            id,
            Entry {
                job: job.clone(),
                cancel: cancel.clone(),
                last_progress_event: None,
            },
        );
        self.emit(job);

        let ctx = JobCtx {
            id,
            manager: self.clone(),
            cancel,
        };
        let manager = self.clone();
        tokio::spawn(async move {
            let state = match run(ctx).await {
                Ok(()) => JobState::Succeeded,
                Err(Error::Cancelled) => JobState::Cancelled,
                Err(err) => JobState::Failed {
                    message: err.to_string(),
                },
            };
            manager.finish(id, state);
        });
        id
    }

    /// Ask a running job to stop. Returns `false` if it isn't running.
    pub fn cancel(&self, id: JobId) -> bool {
        let jobs = self.lock();
        match jobs.get(&id) {
            Some(entry) if entry.job.state == JobState::Running => {
                entry.cancel.cancel();
                true
            }
            _ => false,
        }
    }

    /// Forget a finished job. Returns `false` if it is still running.
    pub fn dismiss(&self, id: JobId) -> bool {
        let mut jobs = self.lock();
        if jobs
            .get(&id)
            .is_some_and(|entry| entry.job.state != JobState::Running)
        {
            jobs.remove(&id);
            true
        } else {
            false
        }
    }

    fn report(&self, id: JobId, progress: JobProgress) {
        let event = {
            let mut jobs = self.lock();
            let Some(entry) = jobs.get_mut(&id) else {
                return;
            };
            entry.job.progress = Some(progress);
            let due = entry
                .last_progress_event
                .is_none_or(|last| last.elapsed() >= PROGRESS_THROTTLE);
            if !due {
                return;
            }
            entry.last_progress_event = Some(Instant::now());
            entry.job.clone()
        };
        self.emit(event);
    }

    fn finish(&self, id: JobId, state: JobState) {
        let event = {
            let mut jobs = self.lock();
            let Some(entry) = jobs.get_mut(&id) else {
                return;
            };
            entry.job.state = state;
            entry.job.finished_at_ms = Some(now_ms());
            entry.job.clone()
        };
        self.emit(event);
    }

    fn emit(&self, job: Job) {
        // No subscribers is fine: the UI may not be listening yet.
        let _ = self.inner.events.send(job);
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<JobId, Entry>> {
        self.inner
            .jobs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Handle given to a running job to report progress and observe cancellation.
#[derive(Debug, Clone)]
pub struct JobCtx {
    id: JobId,
    manager: JobManager,
    cancel: CancellationToken,
}

impl JobCtx {
    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn report(&self, progress: JobProgress) {
        self.manager.report(self.id, progress);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Resolves once cancellation is requested.
    pub async fn cancelled(&self) {
        self.cancel.cancelled().await;
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pull(model: &str) -> JobKind {
        JobKind::Pull {
            model: model.into(),
        }
    }

    async fn wait_until_finished(jobs: &JobManager, id: JobId) -> Job {
        let mut events = jobs.subscribe();
        loop {
            if let Some(job) = jobs.get(id).filter(|job| job.state != JobState::Running) {
                return job;
            }
            let _ = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
        }
    }

    #[tokio::test]
    async fn successful_job_reports_progress_then_succeeds() {
        let jobs = JobManager::new();
        let id = jobs.spawn(pull("a/b"), |ctx| async move {
            ctx.report(JobProgress {
                done_bytes: 5,
                total_bytes: Some(10),
                bytes_per_second: None,
                phase: None,
            });
            Ok(())
        });
        let job = wait_until_finished(&jobs, id).await;
        assert_eq!(job.state, JobState::Succeeded);
        assert_eq!(job.progress.map(|p| p.done_bytes), Some(5));
        assert!(job.finished_at_ms.is_some());
    }

    #[tokio::test]
    async fn errors_mark_the_job_failed() {
        let jobs = JobManager::new();
        let id = jobs.spawn(pull("a/b"), |_| async { Err(Error::Parse("boom".into())) });
        let job = wait_until_finished(&jobs, id).await;
        assert!(matches!(job.state, JobState::Failed { ref message } if message.contains("boom")));
    }

    #[tokio::test]
    async fn cancel_stops_a_running_job() {
        let jobs = JobManager::new();
        let id = jobs.spawn(pull("a/b"), |ctx| async move {
            ctx.cancelled().await;
            Err(Error::Cancelled)
        });
        assert!(jobs.cancel(id));
        let job = wait_until_finished(&jobs, id).await;
        assert_eq!(job.state, JobState::Cancelled);
        assert!(!jobs.cancel(id), "finished jobs can't be cancelled");
    }

    #[tokio::test]
    async fn only_finished_jobs_can_be_dismissed() {
        let jobs = JobManager::new();
        let id = jobs.spawn(pull("a/b"), |ctx| async move {
            ctx.cancelled().await;
            Err(Error::Cancelled)
        });
        assert!(!jobs.dismiss(id));
        jobs.cancel(id);
        wait_until_finished(&jobs, id).await;
        assert!(jobs.dismiss(id));
        assert_eq!(jobs.list(), []);
    }
}
