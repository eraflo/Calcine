//! Usage over time, for the Activity page and `/metrics`: requests, tokens,
//! speed and energy per hour, per app and per model. Counts only, never
//! prompts or replies, kept 90 days in `usage.json`.
//!
//! - `bucket`: the counters of one hour of one app on one model
//! - `store`: the buckets, on disk
//! - `recorder`: follows the request log, reading the energy meter as
//!   requests start and end
//! - `summary`: totals, a time series and shares for a period
//! - `prometheus`: the counters in Prometheus' text format

mod bucket;
mod prometheus;
mod recorder;
mod store;
mod summary;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use calcine_core::hardware::HardwareProbe;
pub use summary::{UsagePeriod, UsagePoint, UsageShare, UsageSummary, UsageTotals};

use crate::log::{RequestEntry, RequestLog};
use crate::now_ms;
use store::UsageStore;

#[derive(Debug)]
pub struct Usage {
    store: Mutex<UsageStore>,
    /// `None` keeps usage in memory (tests).
    path: Option<PathBuf>,
    /// Recorded since the last save.
    changed: AtomicBool,
}

impl Usage {
    /// The usage saved at `path`, if any.
    pub fn load(path: Option<PathBuf>) -> Self {
        let mut store = path.as_deref().map(UsageStore::load).unwrap_or_default();
        store.prune(now_ms());
        Self {
            store: Mutex::new(store),
            path,
            changed: AtomicBool::new(false),
        }
    }

    /// Count the requests of `log` as they finish, with the energy
    /// `hardware` measures. Needs a Tokio runtime; without one nothing is
    /// counted.
    pub fn follow(self: &Arc<Self>, log: &RequestLog, hardware: Arc<dyn HardwareProbe>) {
        if recorder::spawn(self.clone(), log, hardware).is_none() {
            tracing::warn!("no runtime: usage isn't recorded");
        }
    }

    pub fn record(&self, entry: &RequestEntry, energy_joules: Option<f64>) {
        self.lock().record(entry, energy_joules);
        self.changed.store(true, Ordering::Relaxed);
    }

    pub fn summary(&self, period: UsagePeriod, utc_offset_minutes: i32) -> UsageSummary {
        summary::summarize(self.lock().buckets(), period, now_ms(), utc_offset_minutes)
    }

    /// Every counter, in Prometheus' text format.
    pub fn prometheus(&self) -> String {
        prometheus::render(self.lock().buckets())
    }

    /// Forget all usage.
    pub fn clear(&self) {
        self.lock().clear();
        self.changed.store(true, Ordering::Relaxed);
        self.save();
    }

    /// Write recorded usage to disk, if anything changed.
    pub fn save(&self) {
        let Some(path) = &self.path else { return };
        if !self.changed.swap(false, Ordering::Relaxed) {
            return;
        }
        let mut store = self.lock();
        store.prune(now_ms());
        if let Err(err) = store.save(path) {
            tracing::warn!(%err, path = %path.display(), "couldn't save usage");
            self.changed.store(true, Ordering::Relaxed);
        }
    }

    fn lock(&self) -> MutexGuard<'_, UsageStore> {
        self.store.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
