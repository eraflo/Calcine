//! One hour of one app on one model.

use serde::{Deserialize, Serialize};

use crate::log::RequestEntry;

pub(super) const HOUR_MS: u64 = 3_600_000;

/// What a bucket is about.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BucketKey {
    /// Start of the hour, in milliseconds since 1970 (UTC).
    pub hour_ms: u64,
    /// The API key's name, `Calcine`, or `No key`.
    pub client: String,
    pub model: String,
}

/// Counters for a bucket. Sums, so buckets add up over any period.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct Counters {
    pub requests: u64,
    pub errors: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub duration_ms: u64,
    /// Time to first token, summed over the streamed requests that had one.
    pub first_token_ms: u64,
    pub first_token_count: u64,
    /// Generation speed weighted by tokens: Σ(speed × tokens) and Σ tokens,
    /// over the requests that reported a speed.
    pub speed_weighted: f64,
    pub speed_tokens: u64,
    /// System energy while the requests ran, over those that were metered.
    pub energy_joules: f64,
    pub energy_count: u64,
}

impl Counters {
    /// Count a finished request.
    pub fn add_request(&mut self, entry: &RequestEntry, energy_joules: Option<f64>) {
        self.requests += 1;
        let failed = entry.error.is_some() || entry.status.is_some_and(|status| status >= 400);
        if failed {
            self.errors += 1;
        }
        self.prompt_tokens += entry.prompt_tokens.unwrap_or(0);
        self.completion_tokens += entry.completion_tokens.unwrap_or(0);
        self.duration_ms += entry.duration_ms.unwrap_or(0);
        if let Some(first) = entry.first_token_ms {
            self.first_token_ms += first;
            self.first_token_count += 1;
        }
        if let (Some(speed), Some(tokens)) = (entry.tokens_per_second, entry.completion_tokens)
            && speed.is_finite()
            && tokens > 0
        {
            #[allow(clippy::cast_precision_loss)]
            let weight = tokens as f64;
            self.speed_weighted += speed * weight;
            self.speed_tokens += tokens;
        }
        if let Some(joules) = energy_joules.filter(|joules| joules.is_finite() && *joules >= 0.0) {
            self.energy_joules += joules;
            self.energy_count += 1;
        }
    }

    /// Add another bucket's counters.
    pub fn merge(&mut self, other: &Self) {
        self.requests += other.requests;
        self.errors += other.errors;
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.duration_ms += other.duration_ms;
        self.first_token_ms += other.first_token_ms;
        self.first_token_count += other.first_token_count;
        self.speed_weighted += other.speed_weighted;
        self.speed_tokens += other.speed_tokens;
        self.energy_joules += other.energy_joules;
        self.energy_count += other.energy_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(tokens: u64, speed: f64, status: u16) -> RequestEntry {
        let mut entry = RequestEntry::new("App".into(), "POST", "/v1/chat/completions", None, true);
        entry.status = Some(status);
        entry.duration_ms = Some(1000);
        entry.first_token_ms = Some(200);
        entry.completion_tokens = Some(tokens);
        entry.prompt_tokens = Some(10);
        entry.tokens_per_second = Some(speed);
        entry
    }

    #[test]
    fn speed_is_weighted_by_tokens() {
        let mut counters = Counters::default();
        counters.add_request(&entry(100, 20.0, 200), Some(5.0));
        counters.add_request(&entry(300, 40.0, 200), None);
        assert_eq!(counters.requests, 2);
        assert_eq!(counters.completion_tokens, 400);
        assert!((counters.speed_weighted / 400.0 - 35.0).abs() < 1e-9);
        assert_eq!(counters.energy_count, 1);
        assert_eq!(counters.first_token_count, 2);
    }

    #[test]
    fn failures_are_counted() {
        let mut counters = Counters::default();
        counters.add_request(&entry(0, 0.0, 500), None);
        assert_eq!(counters.errors, 1);
        assert_eq!(counters.speed_tokens, 0, "no tokens, no speed");
    }
}
