//! The hourly buckets, kept on disk for [`RETENTION_DAYS`].

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::bucket::{BucketKey, Counters, HOUR_MS};
use crate::log::RequestEntry;

/// Older hours are dropped.
pub(super) const RETENTION_DAYS: u64 = 90;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(super) struct UsageStore {
    buckets: BTreeMap<BucketKey, Counters>,
}

/// On disk: a list, as JSON object keys must be strings.
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    buckets: Vec<(BucketKey, Counters)>,
}

impl UsageStore {
    /// The saved buckets; empty when there's no file or it can't be read.
    pub fn load(path: &Path) -> Self {
        let Ok(json) = std::fs::read(path) else {
            return Self::default();
        };
        match serde_json::from_slice::<Saved>(&json) {
            Ok(saved) => Self {
                buckets: saved.buckets.into_iter().collect(),
            },
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "ignoring unreadable usage");
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let saved = Saved {
            version: 1,
            buckets: self
                .buckets
                .iter()
                .map(|(key, counters)| (key.clone(), counters.clone()))
                .collect(),
        };
        let json = serde_json::to_vec(&saved).map_err(std::io::Error::other)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let partial = path.with_extension("json.partial");
        std::fs::write(&partial, json)?;
        std::fs::rename(partial, path)
    }

    /// Count a finished request in the hour it started.
    pub fn record(&mut self, entry: &RequestEntry, energy_joules: Option<f64>) {
        let key = BucketKey {
            hour_ms: entry.started_at_ms - entry.started_at_ms % HOUR_MS,
            client: entry.client.clone(),
            model: entry.model.clone().unwrap_or_else(|| "—".to_owned()),
        };
        self.buckets
            .entry(key)
            .or_default()
            .add_request(entry, energy_joules);
    }

    /// Drop the hours older than [`RETENTION_DAYS`] before `now_ms`.
    pub fn prune(&mut self, now_ms: u64) {
        let oldest = now_ms.saturating_sub(RETENTION_DAYS * 24 * HOUR_MS);
        self.buckets.retain(|key, _| key.hour_ms >= oldest);
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    pub fn buckets(&self) -> impl Iterator<Item = (&BucketKey, &Counters)> {
        self.buckets.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(started_at_ms: u64, client: &str) -> RequestEntry {
        let mut entry = RequestEntry::new(
            client.into(),
            "POST",
            "/v1/chat/completions",
            Some("qualcomm/Qwen3-4B".into()),
            false,
        );
        entry.started_at_ms = started_at_ms;
        entry.status = Some(200);
        entry.completion_tokens = Some(10);
        entry
    }

    #[test]
    fn requests_of_the_same_hour_share_a_bucket() {
        let mut store = UsageStore::default();
        store.record(&entry(HOUR_MS * 5 + 10, "App"), None);
        store.record(&entry(HOUR_MS * 5 + 3_000_000, "App"), None);
        store.record(&entry(HOUR_MS * 6, "App"), None);
        store.record(&entry(HOUR_MS * 5, "Other"), None);
        let counts: Vec<_> = store
            .buckets()
            .map(|(key, c)| (key.hour_ms / HOUR_MS, c.requests))
            .collect();
        assert_eq!(counts, vec![(5, 2), (5, 1), (6, 1)]);
    }

    #[test]
    fn old_hours_are_dropped() {
        let mut store = UsageStore::default();
        let now = RETENTION_DAYS * 24 * HOUR_MS + 10 * HOUR_MS;
        store.record(&entry(HOUR_MS, "App"), None);
        store.record(&entry(now - HOUR_MS, "App"), None);
        store.prune(now);
        assert_eq!(store.buckets().count(), 1);
    }

    #[test]
    fn usage_round_trips_on_disk() {
        let mut random = [0u8; 8];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!("calcine-usage-{}.json", hex::encode(random)));
        let mut store = UsageStore::default();
        store.record(&entry(HOUR_MS, "App"), Some(12.5));
        store.save(&path).unwrap();
        assert_eq!(UsageStore::load(&path), store);
        std::fs::remove_file(path).ok();
    }
}
