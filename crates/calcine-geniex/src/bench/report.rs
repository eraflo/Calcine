//! Reading what `geniex-bench` writes: the per-run JSON report, and the
//! error it prints when a run fails.

use calcine_core::bench::{BenchMeasure, BenchStat};
use calcine_core::{Error, Result};
use serde_json::Value;

/// The `agg` section of a report (schema 6), as a measure.
pub fn parse(json: &str) -> Result<BenchMeasure> {
    let report: Value = serde_json::from_str(json)
        .map_err(|err| Error::Parse(format!("geniex-bench report isn't JSON: {err}")))?;
    let agg = report
        .get("agg")
        .ok_or_else(|| Error::Parse("geniex-bench report has no results".into()))?;
    let stat = |name: &str| -> Result<BenchStat> {
        let value = agg
            .get(name)
            .ok_or_else(|| Error::Parse(format!("geniex-bench report has no {name}")))?;
        let field = |key: &str| value.get(key).and_then(Value::as_f64).unwrap_or(0.0);
        Ok(BenchStat {
            median: field("median"),
            min: field("min"),
            max: field("max"),
            stdev: field("stdev"),
        })
    };
    let median = |name: &str| {
        agg.pointer(&format!("/{name}/median"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    Ok(BenchMeasure {
        ttft_ms: stat("ttft_ms")?,
        prefill_tps: stat("prefill_tps")?,
        decode_tps: stat("decode_tps")?,
        generated_tokens: median("gen_tokens"),
        prompt_tokens: median("prompt_tokens"),
        geniex_version: report
            .get("geniex_version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        energy: None,
    })
}

/// Why a run failed, from its output: the last `ERROR:` line, or the last
/// warning that explains it (llama.cpp failing to load on a unit).
pub fn failure(output: &str) -> Option<String> {
    let lines: Vec<String> = output.lines().map(strip_ansi).collect();
    let error = lines.iter().rev().find_map(|line| {
        line.trim()
            .strip_prefix("ERROR:")
            .map(|message| message.trim().to_owned())
    })?;
    // `Model loading failed` alone doesn't say why; the warning before does.
    let reason = lines.iter().rev().find_map(|line| {
        let line = line.trim();
        line.contains("failed to initialize the context").then(|| {
            line.rsplit_once("failed to initialize the context:")
                .map_or(line, |(_, reason)| reason.trim())
                .to_owned()
        })
    });
    Some(match reason {
        Some(reason) => format!("{error} ({reason})"),
        None => error,
    })
}

fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip `ESC [ ... letter`.
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_real_report() {
        let measure = parse(include_str!("../../tests/fixtures/bench_report.json")).unwrap();
        assert!((measure.decode_tps.median - 53.072_619).abs() < 1e-6);
        assert!((measure.ttft_ms.min - 36.461).abs() < 1e-6);
        assert!((measure.prefill_tps.stdev - 241.864_635).abs() < 1e-6);
        assert!((measure.generated_tokens - 32.0).abs() < f64::EPSILON);
        assert!((measure.prompt_tokens - 128.0).abs() < f64::EPSILON);
        assert_eq!(measure.geniex_version, "v0.8.0");
    }

    #[test]
    fn rejects_reports_without_results() {
        assert!(parse("{}").is_err());
        assert!(parse("not json").is_err());
    }

    #[test]
    fn finds_the_failure_and_its_reason() {
        let output = "\u{1b}[33m[ WARN] [plugin.cpp:52] llama_init_from_model: failed to initialize the context: ggml-hex: failed to open session (see log for details)\u{1b}[0m\n\
                      ERROR: geniex_llm_create: Model loading failed (code=-100201)\n";
        assert_eq!(
            failure(output).unwrap(),
            "geniex_llm_create: Model loading failed (code=-100201) (ggml-hex: failed to open session (see log for details))"
        );
        assert_eq!(failure("[ok  ] cell ttft=1ms"), None);
    }
}
