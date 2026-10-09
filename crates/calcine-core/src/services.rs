use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Result;
use crate::bench::{
    BenchRequest, BenchResult, BenchSource, Benchmarker, EnergyRequest, measure_mode,
};
use crate::context::{ContextMeter, ContextUsage};
use crate::hardware::HardwareProbe;
use crate::jobs::{JobCtx, JobId, JobKind, JobManager, JobPhase, JobProgress, JobState, JobStep};
use crate::models::{ComputeUnit, ModelCatalog, ModelDirectory, ModelKey, ModelStore, PullRequest};
use crate::runtime::{
    InferenceServer, InstallSource, RuntimeInstaller, RuntimeManager, ServerState,
};

/// Which implementation backs the services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// The real GenieX CLI.
    Geniex,
    /// In-memory fake data (`CALCINE_BACKEND=mock`).
    Mock,
}

/// The single set of services shared by Tauri commands and the HTTP gateway.
#[derive(Clone)]
pub struct Services {
    pub backend: BackendKind,
    pub models: Arc<dyn ModelStore>,
    pub catalog: Arc<dyn ModelCatalog>,
    /// Hugging Face and AI Hub over HTTP.
    pub directory: Arc<dyn ModelDirectory>,
    pub runtime: Arc<dyn RuntimeManager>,
    /// `geniex serve`, started on demand by the gateway.
    pub server: Arc<dyn InferenceServer>,
    pub hardware: Arc<dyn HardwareProbe>,
    /// Installs and updates GenieX itself.
    pub installer: Arc<dyn RuntimeInstaller>,
    /// `geniex-bench` and the benchmark history.
    pub bench: Arc<dyn Benchmarker>,
    /// Context windows and token counts.
    pub context: Arc<dyn ContextMeter>,
    pub jobs: JobManager,
}

impl Services {
    /// Start downloading a model, or return the job already downloading it.
    pub fn start_pull(&self, request: PullRequest) -> JobId {
        let model = request.reference.cli_arg();
        let kind = if request.is_import() {
            JobKind::Import {
                model,
                path: request.local_path.clone().unwrap_or_default(),
            }
        } else {
            JobKind::Pull { model }
        };
        if let Some(job) = self
            .jobs
            .list()
            .into_iter()
            .find(|job| job.kind == kind && job.state == JobState::Running)
        {
            return job.id;
        }
        let store = self.models.clone();
        self.jobs.spawn(
            kind,
            move |ctx| async move { store.pull(request, ctx).await },
        )
    }
}

impl Services {
    /// Install, update or roll back GenieX in the background, or return the
    /// install already running. `geniex serve` is stopped first (the
    /// installer kills it anyway) and started again afterwards if it was
    /// running.
    pub fn start_runtime_install(&self, source: InstallSource) -> JobId {
        if let Some(job) = self.jobs.list().into_iter().find(|job| {
            matches!(job.kind, JobKind::InstallRuntime { .. }) && job.state == JobState::Running
        }) {
            return job.id;
        }
        let kind = JobKind::InstallRuntime {
            version: source.version().to_owned(),
        };
        let installer = self.installer.clone();
        let server = self.server.clone();
        self.jobs.spawn(kind, move |ctx| async move {
            let was_running = matches!(
                *server.state().borrow(),
                ServerState::Ready { .. } | ServerState::Starting
            );
            server.stop().await?;
            let result = installer.install(source, ctx).await;
            if was_running && result.is_ok() {
                let _ = server.ensure_running().await;
            }
            result
        })
    }

    /// Download the benchmark tool, or return the download already running.
    pub fn start_bench_install(&self, version: String) -> JobId {
        if let Some(job) = self.running_job(|kind| matches!(kind, JobKind::InstallBench { .. })) {
            return job;
        }
        let bench = self.bench.clone();
        self.jobs
            .spawn(JobKind::InstallBench { version }, move |ctx| async move {
                bench.install_tool(ctx).await
            })
    }

    /// Benchmark a model on each requested compute unit, one after the
    /// other, then save the results. `geniex serve` is paused meanwhile so
    /// it doesn't share the hardware; a failed unit is recorded with its
    /// error and the others still run.
    pub fn start_benchmark(&self, request: BenchRequest) -> Result<JobId> {
        request.validate().map_err(crate::Error::InvalidInput)?;
        self.ensure_no_benchmark()?;
        let bench = self.bench.clone();
        let server = self.server.clone();
        let kind = JobKind::Benchmark {
            model: request.model.clone(),
        };
        Ok(self.jobs.spawn(kind, move |ctx| async move {
            let was_running = matches!(
                *server.state().borrow(),
                ServerState::Ready { .. } | ServerState::Starting
            );
            server.stop().await?;
            let outcome = measure_all(bench.as_ref(), &request, &ctx).await;
            if was_running {
                let _ = server.ensure_running().await;
            }
            let results = outcome?;
            let all_failed = results.iter().all(|result| result.measure.is_none());
            let first_error = results.iter().find_map(|result| result.error.clone());
            bench.record(results)?;
            match first_error {
                Some(message) if all_failed => Err(crate::Error::Command {
                    command: "geniex-bench".into(),
                    message,
                }),
                _ => Ok(()),
            }
        }))
    }

    /// Measure a model's speed and energy in each requested power mode,
    /// through `geniex serve` (started if needed), then save the results.
    /// Other requests to Calcine's API meanwhile would skew them.
    pub fn start_energy_profile(&self, request: EnergyRequest) -> Result<JobId> {
        request.validate().map_err(crate::Error::InvalidInput)?;
        self.ensure_no_benchmark()?;
        let bench = self.bench.clone();
        let server = self.server.clone();
        let hardware = self.hardware.clone();
        let runtime = self.runtime.clone();
        let kind = JobKind::EnergyProfile {
            model: request.model.clone(),
        };
        Ok(self.jobs.spawn(kind, move |ctx| async move {
            let version = runtime
                .info()
                .await
                .map(|info| info.cli_version)
                .unwrap_or_default();
            let results =
                profile_all(server.as_ref(), hardware.as_ref(), &request, &version, &ctx).await?;
            let all_failed = results.iter().all(|result| result.measure.is_none());
            let first_error = results.iter().find_map(|result| result.error.clone());
            bench.record(results)?;
            match first_error {
                Some(message) if all_failed => Err(crate::Error::Command {
                    command: "geniex serve".into(),
                    message,
                }),
                _ => Ok(()),
            }
        }))
    }

    fn ensure_no_benchmark(&self) -> Result<()> {
        let running = self.running_job(|kind| {
            matches!(
                kind,
                JobKind::Benchmark { .. } | JobKind::EnergyProfile { .. }
            )
        });
        match running {
            Some(_) => Err(crate::Error::InvalidInput(
                "a benchmark is already running".into(),
            )),
            None => Ok(()),
        }
    }

    fn running_job(&self, matches: impl Fn(&JobKind) -> bool) -> Option<JobId> {
        self.jobs
            .list()
            .into_iter()
            .find(|job| matches(&job.kind) && job.state == JobState::Running)
            .map(|job| job.id)
    }

    /// A model's context window: compiled into AI Hub models, the server
    /// option for llama.cpp ones.
    pub async fn context_window(&self, model: &str) -> Result<u32> {
        Ok(match self.context.compiled_window(model).await? {
            Some(window) => window,
            None => self.server.options().context_size,
        })
    }

    /// How much of `model`'s context window a chat request takes.
    pub async fn context_usage(
        &self,
        model: &str,
        request: &serde_json::Value,
    ) -> Result<ContextUsage> {
        Ok(ContextUsage {
            tokens: self.context.count(model, request).await?,
            window: Some(self.context_window(model).await?),
        })
    }

    /// Delete models or precisions. Stops `geniex serve` first: Windows
    /// can't delete the files of a loaded model.
    pub async fn remove_models(&self, keys: &[ModelKey]) -> Result<()> {
        self.server.stop().await?;
        self.models.remove(keys).await
    }

    /// Delete every cached model, stopping `geniex serve` first.
    pub async fn clean_models(&self) -> Result<()> {
        self.server.stop().await?;
        self.models.clean().await
    }
}

/// One measurement per unit and power mode. Stops early when cancelled, or when the
/// device can't measure energy at all.
async fn profile_all(
    server: &dyn InferenceServer,
    hardware: &dyn HardwareProbe,
    request: &EnergyRequest,
    version: &str,
    ctx: &JobCtx,
) -> Result<Vec<BenchResult>> {
    let session_id = format!("{:x}-{}", now_ms(), ctx.id());
    let pairs: Vec<(ComputeUnit, &String)> = request
        .units
        .iter()
        .flat_map(|unit| request.power_modes.iter().map(move |mode| (*unit, mode)))
        .collect();
    let total = u32::try_from(pairs.len()).unwrap_or(u32::MAX);
    let mut results = Vec::with_capacity(pairs.len());
    for (index, (unit, mode)) in pairs.into_iter().enumerate() {
        ctx.report(JobProgress {
            phase: Some(JobPhase::Measuring),
            step: Some(JobStep {
                current: u32::try_from(index + 1).unwrap_or(u32::MAX),
                total,
            }),
            ..JobProgress::default()
        });
        let started_at_ms = now_ms();
        let (measure, error) = match measure_mode(server, hardware, request, unit, mode, ctx).await
        {
            Ok(mut measure) => {
                version.clone_into(&mut measure.geniex_version);
                (Some(measure), None)
            }
            Err(err @ (crate::Error::Cancelled | crate::Error::NotImplemented(_))) => {
                return Err(err);
            }
            Err(err) => (None, Some(err.to_string())),
        };
        results.push(BenchResult {
            id: format!("{session_id}-{index}"),
            session_id: session_id.clone(),
            started_at_ms,
            model: request.model.clone(),
            runtime: request.runtime,
            unit,
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            prompt_tokens: measure
                .as_ref()
                .map_or(0, |measure| measure.prompt_tokens as u32),
            generated_tokens: request.generated_tokens,
            repetitions: request.repetitions,
            power_mode: mode.clone(),
            spec_type: None,
            draft_model: None,
            source: BenchSource::EnergyProfile,
            measure,
            error,
        });
    }
    Ok(results)
}

/// One measurement per compute unit. Stops early only when cancelled.
async fn measure_all(
    bench: &dyn Benchmarker,
    request: &BenchRequest,
    ctx: &JobCtx,
) -> Result<Vec<BenchResult>> {
    let session_id = format!("{:x}-{}", now_ms(), ctx.id());
    let total = u32::try_from(request.units.len()).unwrap_or(u32::MAX);
    let mut results = Vec::with_capacity(request.units.len());
    for (index, unit) in request.units.iter().enumerate() {
        let current = u32::try_from(index + 1).unwrap_or(u32::MAX);
        ctx.report(JobProgress {
            phase: Some(JobPhase::Measuring),
            step: Some(JobStep { current, total }),
            ..JobProgress::default()
        });
        let started_at_ms = now_ms();
        let outcome = bench.measure(request, *unit, ctx).await;
        if matches!(outcome, Err(crate::Error::Cancelled)) {
            return Err(crate::Error::Cancelled);
        }
        let (measure, error) = match outcome {
            Ok(measure) => (Some(measure), None),
            Err(err) => (None, Some(err.to_string())),
        };
        results.push(BenchResult {
            id: format!("{session_id}-{index}"),
            session_id: session_id.clone(),
            started_at_ms,
            model: request.model.clone(),
            runtime: request.runtime,
            unit: *unit,
            prompt_tokens: request.prompt_tokens,
            generated_tokens: request.generated_tokens,
            repetitions: request.repetitions,
            power_mode: request.power_mode.clone(),
            spec_type: request.spec_type.clone(),
            draft_model: request.draft_model.clone(),
            source: BenchSource::GeniexBench,
            measure,
            error,
        });
    }
    Ok(results)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

impl fmt::Debug for Services {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Services")
            .field("backend", &self.backend)
            .finish_non_exhaustive()
    }
}
