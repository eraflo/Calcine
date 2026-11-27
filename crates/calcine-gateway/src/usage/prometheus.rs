//! The counters in Prometheus' text format, for `/metrics` (Grafana and
//! other monitoring tools). One series per app and model, summed over the
//! kept history; a counter going down means old hours were dropped, which
//! Prometheus treats as a reset.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::bucket::{BucketKey, Counters};

/// (name, help, value) for each metric.
type Metric = (&'static str, &'static str, fn(&Counters) -> f64);

#[allow(clippy::cast_precision_loss)]
const METRICS: &[Metric] = &[
    (
        "calcine_requests_total",
        "Inference requests answered.",
        |c| c.requests as f64,
    ),
    (
        "calcine_request_errors_total",
        "Requests that failed.",
        |c| c.errors as f64,
    ),
    (
        "calcine_prompt_tokens_total",
        "Tokens read from prompts.",
        |c| c.prompt_tokens as f64,
    ),
    (
        "calcine_completion_tokens_total",
        "Tokens generated.",
        |c| c.completion_tokens as f64,
    ),
    (
        "calcine_request_duration_seconds_total",
        "Time spent answering requests.",
        |c| c.duration_ms as f64 / 1000.0,
    ),
    (
        "calcine_energy_joules_total",
        "System energy while requests ran, on devices with energy metering.",
        |c| c.energy_joules,
    ),
];

pub(super) fn render<'a>(buckets: impl Iterator<Item = (&'a BucketKey, &'a Counters)>) -> String {
    let mut series: BTreeMap<(&str, &str), Counters> = BTreeMap::new();
    for (key, counters) in buckets {
        series
            .entry((&key.client, &key.model))
            .or_default()
            .merge(counters);
    }
    let mut out = String::new();
    for (name, help, value) in METRICS {
        let _ = writeln!(out, "# HELP {name} {help}");
        let _ = writeln!(out, "# TYPE {name} counter");
        for ((client, model), counters) in &series {
            let _ = writeln!(
                out,
                "{name}{{client=\"{}\",model=\"{}\"}} {}",
                label(client),
                label(model),
                value(counters)
            );
        }
    }
    out
}

/// A label value, escaped as the format requires.
fn label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_series_per_app_and_model() {
        let key = |hour, client: &str| BucketKey {
            hour_ms: hour,
            client: client.into(),
            model: "m".into(),
        };
        let counters = Counters {
            requests: 2,
            completion_tokens: 30,
            ..Counters::default()
        };
        let buckets = [
            (key(0, "App \"1\""), counters.clone()),
            (key(1, "App \"1\""), counters),
        ];
        let text = render(buckets.iter().map(|(k, c)| (k, c)));
        assert!(text.contains("# TYPE calcine_requests_total counter\n"));
        assert!(
            text.contains("calcine_requests_total{client=\"App \\\"1\\\"\",model=\"m\"} 4\n"),
            "{text}"
        );
        assert!(text.contains(
            "calcine_completion_tokens_total{client=\"App \\\"1\\\"\",model=\"m\"} 60\n"
        ));
    }
}
