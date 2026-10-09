use calcine_core::Services;
use calcine_core::bench::{BenchRequest, BenchResult, BenchTool, EnergyRequest};
use calcine_core::jobs::JobId;
use tauri::State;

use crate::ipc::error::{ApiError, ApiResult};

/// Whether `geniex-bench` is downloaded, and for which GenieX.
#[tauri::command]
#[specta::specta]
pub async fn bench_tool(services: State<'_, Services>) -> ApiResult<BenchTool> {
    Ok(services.bench.tool().await?)
}

/// Download `geniex-bench` for the installed GenieX, as a job.
#[tauri::command]
#[specta::specta]
pub async fn install_bench_tool(services: State<'_, Services>) -> ApiResult<JobId> {
    let tool = services.bench.tool().await?;
    let version = tool
        .wanted
        .ok_or_else(|| ApiError::invalid_input("install GenieX first".into()))?;
    Ok(services.start_bench_install(version))
}

/// Benchmark a model on each requested compute unit, as a job.
#[tauri::command]
#[specta::specta]
pub async fn start_benchmark(
    services: State<'_, Services>,
    request: BenchRequest,
) -> ApiResult<JobId> {
    Ok(services.start_benchmark(request)?)
}

/// Measure a model's speed and energy in each requested power mode, as a
/// job. Needs a device with energy metering.
#[tauri::command]
#[specta::specta]
pub async fn start_energy_profile(
    services: State<'_, Services>,
    request: EnergyRequest,
) -> ApiResult<JobId> {
    Ok(services.start_energy_profile(request)?)
}

/// Past results, newest first.
#[tauri::command]
#[specta::specta]
pub fn bench_history(services: State<'_, Services>) -> Vec<BenchResult> {
    services.bench.history()
}

#[tauri::command]
#[specta::specta]
pub fn forget_bench_results(services: State<'_, Services>, ids: Vec<String>) -> ApiResult<()> {
    Ok(services.bench.forget(&ids)?)
}
