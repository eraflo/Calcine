//! Benchmarks with `geniex-bench`: time to first token, prefill and decode
//! speed of a model on each compute unit, measured with warmup and
//! repetitions on a fixed random prompt, and kept as history.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Result;
use crate::jobs::JobCtx;
use crate::models::{ComputeUnit, Runtime};

/// Whether the benchmark tool is ready.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchTool {
    /// Version installed for Calcine, if any.
    pub installed: Option<String>,
    /// The version matching the installed GenieX, the one to install.
    pub wanted: Option<String>,
    /// Download size of `wanted`, when known.
    pub download_bytes: Option<u64>,
}

impl BenchTool {
    pub fn is_ready(&self) -> bool {
        self.installed.is_some() && self.installed == self.wanted
    }
}

/// What to measure. One run per compute unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchRequest {
    /// GenieX id, `org/model:precision`.
    pub model: String,
    pub runtime: Runtime,
    pub units: Vec<ComputeUnit>,
    /// Prompt length, in random tokens.
    pub prompt_tokens: u32,
    /// Tokens generated per repetition.
    pub generated_tokens: u32,
    /// Measured repetitions, after one warmup.
    pub repetitions: u32,
    /// HTP power mode (`burst`, `balanced`, …).
    pub power_mode: String,
}

impl BenchRequest {
    /// Why this request can't run, if it can't.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.model.trim().is_empty() {
            return Err("choose a model".into());
        }
        if self.units.is_empty() {
            return Err("choose at least one compute unit".into());
        }
        if !(16..=8192).contains(&self.prompt_tokens) {
            return Err("prompt length must be between 16 and 8192 tokens".into());
        }
        if !(8..=2048).contains(&self.generated_tokens) {
            return Err("generate between 8 and 2048 tokens".into());
        }
        if !(1..=20).contains(&self.repetitions) {
            return Err("run between 1 and 20 repetitions".into());
        }
        Ok(())
    }
}

/// A measured value across repetitions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchStat {
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub stdev: f64,
}

/// What one compute unit measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchMeasure {
    pub ttft_ms: BenchStat,
    pub prefill_tps: BenchStat,
    pub decode_tps: BenchStat,
    /// Median tokens actually generated (a model may stop early).
    pub generated_tokens: f64,
    /// Prompt tokens processed. QAIRT pads them to a multiple of 128.
    pub prompt_tokens: f64,
    pub geniex_version: String,
}

/// One compute unit's result in the history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchResult {
    pub id: String,
    /// Results measured together share a session.
    pub session_id: String,
    pub started_at_ms: u64,
    pub model: String,
    pub runtime: Runtime,
    pub unit: ComputeUnit,
    pub prompt_tokens: u32,
    pub generated_tokens: u32,
    pub repetitions: u32,
    pub power_mode: String,
    /// `None` when the run failed.
    pub measure: Option<BenchMeasure>,
    pub error: Option<String>,
}

/// Runs `geniex-bench` and keeps the results.
#[async_trait]
pub trait Benchmarker: Send + Sync {
    /// Installed and wanted tool versions.
    async fn tool(&self) -> Result<BenchTool>;

    /// Download, verify and unpack the tool for the installed GenieX.
    async fn install_tool(&self, ctx: JobCtx) -> Result<()>;

    /// Measure one compute unit. Cancellable.
    async fn measure(
        &self,
        request: &BenchRequest,
        unit: ComputeUnit,
        ctx: &JobCtx,
    ) -> Result<BenchMeasure>;

    /// Past results, newest first.
    fn history(&self) -> Vec<BenchResult>;

    /// Add results to the history.
    fn record(&self, results: Vec<BenchResult>) -> Result<()>;

    /// Forget results by id.
    fn forget(&self, ids: &[String]) -> Result<()>;
}

/// Results kept in the history.
pub const KEPT_RESULTS: usize = 500;

/// Newest first, at most [`KEPT_RESULTS`].
pub fn merge_history(mut history: Vec<BenchResult>, results: Vec<BenchResult>) -> Vec<BenchResult> {
    let mut merged = results;
    merged.append(&mut history);
    merged.sort_by_key(|result| std::cmp::Reverse(result.started_at_ms));
    merged.truncate(KEPT_RESULTS);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> BenchRequest {
        BenchRequest {
            model: "qualcomm/Qwen3-0.6B:W4A16".into(),
            runtime: Runtime::Qairt,
            units: vec![ComputeUnit::Npu],
            prompt_tokens: 512,
            generated_tokens: 128,
            repetitions: 5,
            power_mode: "burst".into(),
        }
    }

    #[test]
    fn validates_requests() {
        assert!(request().validate().is_ok());
        assert!(
            BenchRequest {
                units: vec![],
                ..request()
            }
            .validate()
            .is_err()
        );
        assert!(
            BenchRequest {
                repetitions: 0,
                ..request()
            }
            .validate()
            .is_err()
        );
        assert!(
            BenchRequest {
                prompt_tokens: 1,
                ..request()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn keeps_the_newest_results() {
        let result = |id: &str, at: u64| BenchResult {
            id: id.into(),
            session_id: "s".into(),
            started_at_ms: at,
            model: "m".into(),
            runtime: Runtime::Qairt,
            unit: ComputeUnit::Npu,
            prompt_tokens: 512,
            generated_tokens: 128,
            repetitions: 5,
            power_mode: "burst".into(),
            measure: None,
            error: None,
        };
        let merged = merge_history(vec![result("old", 1)], vec![result("new", 2)]);
        assert_eq!(merged[0].id, "new");
        assert_eq!(merged.len(), 2);
    }
}
