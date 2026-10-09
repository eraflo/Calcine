//! Simulated benchmarks: a fake tool download, then plausible numbers per
//! compute unit (the NPU fastest to the first token, the GPU close behind on
//! decode), with a little noise and a sample history.

use std::sync::{MutexGuard, PoisonError};

use async_trait::async_trait;
use calcine_core::bench::{
    BenchMeasure, BenchRequest, BenchResult, BenchStat, BenchTool, Benchmarker, merge_history,
};
use calcine_core::jobs::{JobCtx, JobPhase, JobProgress};
use calcine_core::models::{ComputeUnit, Runtime};
use calcine_core::{Error, Result};

use crate::MockBackend;

const TOOL_BYTES: u64 = 84_772_012;
const STEPS: u64 = 12;

impl MockBackend {
    fn bench_tool(&self) -> MutexGuard<'_, Option<String>> {
        self.bench_tool
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn bench_history(&self) -> MutexGuard<'_, Vec<BenchResult>> {
        self.bench_history
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl Benchmarker for MockBackend {
    async fn tool(&self) -> Result<BenchTool> {
        let installed = self.bench_tool().clone();
        let wanted = self.cli_version();
        Ok(BenchTool {
            download_bytes: (installed.as_ref() != Some(&wanted)).then_some(TOOL_BYTES),
            installed,
            wanted: Some(wanted),
        })
    }

    async fn install_tool(&self, ctx: JobCtx) -> Result<()> {
        for step in 1..=STEPS {
            ctx.report(JobProgress {
                done_bytes: TOOL_BYTES * step / STEPS,
                total_bytes: Some(TOOL_BYTES),
                bytes_per_second: Some(TOOL_BYTES / 3),
                phase: Some(JobPhase::Downloading),
                step: None,
            });
            tokio::select! {
                () = tokio::time::sleep(self.pull_duration / 24) => {}
                () = ctx.cancelled() => return Err(Error::Cancelled),
            }
        }
        ctx.report(JobProgress {
            phase: Some(JobPhase::Installing),
            ..JobProgress::default()
        });
        tokio::time::sleep(self.pull_duration / 12).await;
        *self.bench_tool() = Some(self.cli_version());
        Ok(())
    }

    async fn measure(
        &self,
        request: &BenchRequest,
        unit: ComputeUnit,
        ctx: &JobCtx,
    ) -> Result<BenchMeasure> {
        if self.bench_tool().is_none() {
            return Err(Error::InvalidInput(
                "download the benchmark tool first".into(),
            ));
        }
        tokio::select! {
            () = tokio::time::sleep(self.pull_duration / 3) => {}
            () = ctx.cancelled() => return Err(Error::Cancelled),
        }
        if request.runtime == Runtime::Qairt && unit != ComputeUnit::Npu {
            return Err(Error::Command {
                command: "geniex-bench".into(),
                message:
                    "geniex_llm_create: Model loading failed (AI Hub models only run on the NPU)"
                        .into(),
            });
        }
        Ok(sample(
            request,
            unit,
            self.started.elapsed().subsec_millis(),
        ))
    }

    fn history(&self) -> Vec<BenchResult> {
        self.bench_history().clone()
    }

    fn record(&self, results: Vec<BenchResult>) -> Result<()> {
        let mut history = self.bench_history();
        let merged = merge_history(std::mem::take(&mut *history), results);
        *history = merged;
        Ok(())
    }

    fn forget(&self, ids: &[String]) -> Result<()> {
        self.bench_history()
            .retain(|result| !ids.contains(&result.id));
        Ok(())
    }
}

/// Numbers shaped like a Snapdragon X Elite, scaled by model size.
fn sample(request: &BenchRequest, unit: ComputeUnit, seed: u32) -> BenchMeasure {
    let billions = if request.model.contains("4B") {
        4.0
    } else {
        0.6
    };
    let (ttft_per_k, prefill, decode) = match unit {
        ComputeUnit::Npu => (80.0, 3300.0, 53.0),
        ComputeUnit::Hybrid => (120.0, 2100.0, 48.0),
        ComputeUnit::Gpu => (160.0, 2400.0, 45.0),
        ComputeUnit::Cpu => (900.0, 380.0, 24.0),
    };
    let size: f64 = billions / 0.6;
    let noise = 1.0 + f64::from(seed % 7) / 100.0;
    // Speculative decoding helps generation a little in the demo.
    let decode = if request.spec_type.is_some() {
        decode * 1.25
    } else {
        decode
    };
    let stat = |median: f64| BenchStat {
        median,
        min: median * 0.96,
        max: median * 1.05,
        stdev: median * 0.02,
    };
    let prompt = f64::from(request.prompt_tokens);
    BenchMeasure {
        ttft_ms: stat(ttft_per_k * prompt / 1000.0 * size.sqrt() * noise),
        prefill_tps: stat(prefill / size.sqrt() / noise),
        decode_tps: stat(decode / size / noise),
        generated_tokens: f64::from(request.generated_tokens),
        prompt_tokens: prompt,
        geniex_version: "v0.8.0".into(),
        energy: None,
    }
}
