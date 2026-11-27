//! Totals, a time series and shares for a period, from the hourly buckets.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::bucket::{BucketKey, Counters, HOUR_MS};

const DAY_MS: u64 = 24 * HOUR_MS;

/// The span the Activity page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UsagePeriod {
    /// The last 24 hours, hour by hour.
    Day,
    /// The last 7 days, day by day.
    Week,
    /// The last 30 days, day by day.
    Month,
}

impl UsagePeriod {
    /// Points in the series, and how long each covers.
    fn shape(self) -> (u64, u64) {
        match self {
            Self::Day => (24, HOUR_MS),
            Self::Week => (7, DAY_MS),
            Self::Month => (30, DAY_MS),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    pub requests: u64,
    pub errors: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    /// Generated tokens per second, weighted by tokens.
    pub average_speed: Option<f64>,
    pub average_first_token_ms: Option<f64>,
    /// System energy while requests ran; `None` without energy metering.
    pub energy_joules: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UsagePoint {
    pub start_ms: u64,
    pub requests: u64,
    pub completion_tokens: u64,
    pub errors: u64,
    pub energy_joules: Option<f64>,
}

/// One app's or model's part of the period.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UsageShare {
    pub name: String,
    pub requests: u64,
    pub completion_tokens: u64,
    pub energy_joules: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub period: UsagePeriod,
    /// What each point of `series` covers.
    pub step_ms: u64,
    pub start_ms: u64,
    pub end_ms: u64,
    pub totals: UsageTotals,
    /// The same span just before, for trends.
    pub previous: UsageTotals,
    pub series: Vec<UsagePoint>,
    /// Most requests first.
    pub by_client: Vec<UsageShare>,
    pub by_model: Vec<UsageShare>,
}

impl From<&Counters> for UsageTotals {
    fn from(counters: &Counters) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let ratio = |sum: f64, count: u64| (count > 0).then(|| sum / count as f64);
        #[allow(clippy::cast_precision_loss)]
        let first_token_ms = counters.first_token_ms as f64;
        Self {
            requests: counters.requests,
            errors: counters.errors,
            prompt_tokens: counters.prompt_tokens,
            completion_tokens: counters.completion_tokens,
            average_speed: ratio(counters.speed_weighted, counters.speed_tokens),
            average_first_token_ms: ratio(first_token_ms, counters.first_token_count),
            energy_joules: (counters.energy_count > 0).then_some(counters.energy_joules),
        }
    }
}

/// Summarize `buckets` for `period`, ending now. `utc_offset_minutes` puts
/// day boundaries at local midnight.
pub(super) fn summarize<'a>(
    buckets: impl Iterator<Item = (&'a BucketKey, &'a Counters)>,
    period: UsagePeriod,
    now_ms: u64,
    utc_offset_minutes: i32,
) -> UsageSummary {
    let (points, step_ms) = period.shape();
    // The current step ends the series: the current hour, or today (local).
    let offset_ms = i64::from(utc_offset_minutes) * 60_000;
    let local_now = now_ms.saturating_add_signed(offset_ms);
    let current = local_now - local_now % step_ms;
    let start_ms = current
        .saturating_sub((points - 1) * step_ms)
        .saturating_add_signed(-offset_ms);
    let end_ms = start_ms + points * step_ms;
    let previous_start = start_ms.saturating_sub(points * step_ms);

    let mut totals = Counters::default();
    let mut previous = Counters::default();
    let mut series: Vec<Counters> = vec![Counters::default(); usize::try_from(points).unwrap_or(0)];
    let mut clients: HashMap<&str, Counters> = HashMap::new();
    let mut models: HashMap<&str, Counters> = HashMap::new();
    for (key, counters) in buckets {
        if key.hour_ms >= previous_start && key.hour_ms < start_ms {
            previous.merge(counters);
        }
        if key.hour_ms < start_ms || key.hour_ms >= end_ms {
            continue;
        }
        totals.merge(counters);
        let index = usize::try_from((key.hour_ms - start_ms) / step_ms).unwrap_or(usize::MAX);
        if let Some(point) = series.get_mut(index) {
            point.merge(counters);
        }
        clients.entry(&key.client).or_default().merge(counters);
        models.entry(&key.model).or_default().merge(counters);
    }

    UsageSummary {
        period,
        step_ms,
        start_ms,
        end_ms,
        totals: UsageTotals::from(&totals),
        previous: UsageTotals::from(&previous),
        series: series
            .iter()
            .zip(0..)
            .map(|(counters, index)| UsagePoint {
                start_ms: start_ms + index * step_ms,
                requests: counters.requests,
                completion_tokens: counters.completion_tokens,
                errors: counters.errors,
                energy_joules: (counters.energy_count > 0).then_some(counters.energy_joules),
            })
            .collect(),
        by_client: shares(clients),
        by_model: shares(models),
    }
}

fn shares(groups: HashMap<&str, Counters>) -> Vec<UsageShare> {
    let mut shares: Vec<UsageShare> = groups
        .into_iter()
        .map(|(name, counters)| UsageShare {
            name: name.to_owned(),
            requests: counters.requests,
            completion_tokens: counters.completion_tokens,
            energy_joules: (counters.energy_count > 0).then_some(counters.energy_joules),
        })
        .collect();
    shares.sort_by(|a, b| {
        b.requests
            .cmp(&a.requests)
            .then_with(|| a.name.cmp(&b.name))
    });
    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket(hour: u64, client: &str, requests: u64, tokens: u64) -> (BucketKey, Counters) {
        (
            BucketKey {
                hour_ms: hour * HOUR_MS,
                client: client.into(),
                model: "qualcomm/Qwen3-4B".into(),
            },
            Counters {
                requests,
                completion_tokens: tokens,
                ..Counters::default()
            },
        )
    }

    #[test]
    fn the_last_24_hours_hour_by_hour() {
        let now = 100 * HOUR_MS + 1_000;
        let buckets = [
            bucket(100, "App", 2, 20),
            bucket(77, "App", 1, 10),
            bucket(76, "App", 5, 50),
            bucket(60, "Calcine", 4, 40),
        ];
        let summary = summarize(
            buckets.iter().map(|(k, c)| (k, c)),
            UsagePeriod::Day,
            now,
            0,
        );
        assert_eq!(summary.series.len(), 24);
        assert_eq!(summary.start_ms, 77 * HOUR_MS);
        assert_eq!(summary.totals.requests, 3);
        assert_eq!(summary.series[0].requests, 1);
        assert_eq!(summary.series[23].requests, 2);
        assert_eq!(summary.previous.requests, 9, "hours 53 to 76");
        assert_eq!(summary.by_client.len(), 1);
    }

    #[test]
    fn days_start_at_local_midnight() {
        // 2 a.m. UTC is 4 a.m. in UTC+2: today started at 22:00 UTC.
        let now = 10 * DAY_MS + 2 * HOUR_MS;
        let buckets = [
            bucket(10 * 24 - 3, "App", 1, 1),
            bucket(10 * 24 - 1, "App", 1, 1),
        ];
        let summary = summarize(
            buckets.iter().map(|(k, c)| (k, c)),
            UsagePeriod::Week,
            now,
            120,
        );
        let today = summary.series.last().unwrap();
        assert_eq!(today.start_ms, 10 * DAY_MS - 2 * HOUR_MS);
        assert_eq!(today.requests, 1, "22:00 UTC is midnight in UTC+2");
    }

    #[test]
    fn shares_are_sorted_by_requests() {
        let now = 10 * HOUR_MS;
        let buckets = [bucket(9, "Small", 1, 1), bucket(9, "Big", 5, 1)];
        let summary = summarize(
            buckets.iter().map(|(k, c)| (k, c)),
            UsagePeriod::Day,
            now,
            0,
        );
        let names: Vec<_> = summary.by_client.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Big", "Small"]);
    }
}
