//! `geniex-bench`, Qualcomm's benchmark tool, run for Calcine.
//!
//! - `release`: finding the tool in GenieX's GitHub release, with checksums
//! - `report`: reading its JSON report and its errors
//!
//! The tool is a standalone archive (its own `geniex.dll` and runtimes),
//! downloaded on demand into Calcine's data folder at the version of the
//! installed GenieX. Each measurement runs in its own process: in one
//! process, a failure on one compute unit stops the rest.

mod release;
mod report;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use calcine_core::bench::{
    BenchMeasure, BenchRequest, BenchResult, BenchTool, Benchmarker, merge_history,
};
use calcine_core::jobs::{JobCtx, JobPhase, JobProgress};
use calcine_core::models::{ComputeUnit, Runtime};
use calcine_core::runtime::RuntimeManager;
use calcine_core::{Error, Result};
use tokio::process::Command;

use crate::Geniex;
use crate::update::download;

/// A large model with many repetitions takes minutes; this is a backstop.
const RUN_TIMEOUT: Duration = Duration::from_mins(30);

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Where the tool and the history live.
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Unpacked tool versions, downloads and run reports.
    pub tools_dir: PathBuf,
    /// Benchmark history (`benchmarks.json`).
    pub history_path: PathBuf,
}

/// [`Benchmarker`] running the official `geniex-bench`.
#[derive(Debug)]
pub struct GeniexBench {
    geniex: Geniex,
    config: BenchConfig,
    http: reqwest::Client,
    history: Mutex<Vec<BenchResult>>,
}

impl GeniexBench {
    pub fn new(geniex: Geniex, config: BenchConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            // GitHub's API refuses requests without a user agent.
            .user_agent(concat!("Calcine/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|err| Error::Network(err.to_string()))?;
        let history = load_history(&config.history_path);
        Ok(Self {
            geniex,
            config,
            http,
            history: Mutex::new(history),
        })
    }

    /// The GenieX version the tool must match.
    async fn wanted(&self) -> Result<String> {
        Ok(self.geniex.info().await?.cli_version)
    }

    /// Unpacked versions, by folder name `geniex-bench-windows-arm64-<v>`.
    fn installed(&self) -> Vec<(String, PathBuf)> {
        let Ok(entries) = std::fs::read_dir(&self.config.tools_dir) else {
            return Vec::new();
        };
        let prefix = release::archive_name("");
        let prefix = prefix.trim_end_matches(".zip");
        let mut versions: Vec<(String, PathBuf)> = entries
            .filter_map(std::result::Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let version = name.strip_prefix(prefix)?.to_owned();
                let binary = entry.path().join("bin").join("geniex-bench.exe");
                binary.is_file().then_some((version, binary))
            })
            .collect();
        versions.sort_by(|a, b| calcine_core::runtime::compare_versions(&b.0, &a.0));
        versions
    }

    /// The tool to run: the one matching GenieX, else the newest.
    async fn binary(&self) -> Result<PathBuf> {
        let installed = self.installed();
        let wanted = self.wanted().await.ok();
        installed
            .iter()
            .find(|(version, _)| Some(version) == wanted.as_ref())
            .or_else(|| installed.first())
            .map(|(_, binary)| binary.clone())
            .ok_or_else(|| Error::InvalidInput("download the benchmark tool first".into()))
    }

    async fn release(
        &self,
        version: &str,
    ) -> Result<(calcine_core::runtime::InstallerAsset, Option<String>)> {
        let json: serde_json::Value = self
            .http
            .get(format!("{}/releases/tags/{version}", release::RELEASES_API))
            .header("Accept", "application/vnd.github+json")
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| Error::Network(format!("couldn't read the GenieX release: {err}")))?
            .json()
            .await
            .map_err(|err| Error::Parse(format!("unexpected GenieX release: {err}")))?;
        release::find(&json, version)
    }

    fn lock(&self) -> MutexGuard<'_, Vec<BenchResult>> {
        self.history.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn save(&self, history: &[BenchResult]) -> Result<()> {
        let path = &self.config.history_path;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(history).map_err(std::io::Error::other)?;
        let partial = path.with_extension("json.partial");
        std::fs::write(&partial, json)?;
        std::fs::rename(&partial, path)?;
        Ok(())
    }
}

#[async_trait]
impl Benchmarker for GeniexBench {
    async fn tool(&self) -> Result<BenchTool> {
        let wanted = self.wanted().await.ok();
        let installed = self.installed();
        let current = installed
            .iter()
            .find(|(version, _)| Some(version) == wanted.as_ref())
            .or_else(|| installed.first())
            .map(|(version, _)| version.clone());
        let download_bytes = match &wanted {
            Some(version) if current.as_ref() != Some(version) => self
                .release(version)
                .await
                .ok()
                .map(|(asset, _)| asset.size),
            _ => None,
        };
        Ok(BenchTool {
            installed: current,
            wanted,
            download_bytes,
        })
    }

    async fn install_tool(&self, ctx: JobCtx) -> Result<()> {
        if !cfg!(windows) {
            return Err(Error::InvalidInput(
                "benchmarks need Windows on Arm for now".into(),
            ));
        }
        let version = self.wanted().await?;
        let (asset, checksum_url) = self.release(&version).await?;
        // Qualcomm's .sha256 file and GitHub's digest must agree.
        if let Some(url) = checksum_url {
            let text = self
                .http
                .get(url)
                .timeout(Duration::from_secs(20))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|err| Error::Network(format!("couldn't read the checksum: {err}")))?
                .text()
                .await
                .map_err(|err| Error::Network(err.to_string()))?;
            if release::checksum_file_hash(&text).as_deref() != Some(asset.sha256.as_str()) {
                return Err(Error::InvalidInput(format!(
                    "the published checksums of {} don't agree; nothing was installed",
                    asset.name
                )));
            }
        }

        let tools = &self.config.tools_dir;
        let archive = tools.join("downloads").join(&asset.name);
        download::fetch(&self.http, &asset, &archive, &ctx).await?;

        ctx.report(JobProgress {
            phase: Some(JobPhase::Installing),
            ..JobProgress::default()
        });
        let folder = asset.name.trim_end_matches(".zip").to_owned();
        let staging = tools.join(format!(".staging-{version}"));
        let _ = tokio::fs::remove_dir_all(&staging).await;
        tokio::fs::create_dir_all(&staging).await?;
        extract(&archive, &staging).await?;
        let unpacked = staging.join(&folder);
        if !unpacked.join("bin").join("geniex-bench.exe").is_file() {
            let _ = tokio::fs::remove_dir_all(&staging).await;
            return Err(Error::Parse(format!(
                "{} doesn't contain geniex-bench.exe",
                asset.name
            )));
        }
        let target = tools.join(&folder);
        let _ = tokio::fs::remove_dir_all(&target).await;
        tokio::fs::rename(&unpacked, &target).await?;
        let _ = tokio::fs::remove_dir_all(&staging).await;
        let _ = tokio::fs::remove_file(&archive).await;
        // Only removed when empty: another download may be in there.
        if let Some(downloads) = archive.parent() {
            let _ = tokio::fs::remove_dir(downloads).await;
        }
        // Older versions only take space (a few hundred MB each).
        for (old, binary) in self.installed() {
            if old != version
                && let Some(dir) = binary.parent().and_then(Path::parent)
            {
                let _ = tokio::fs::remove_dir_all(dir).await;
            }
        }
        Ok(())
    }

    async fn measure(
        &self,
        request: &BenchRequest,
        unit: ComputeUnit,
        ctx: &JobCtx,
    ) -> Result<BenchMeasure> {
        let binary = self.binary().await?;
        let plugin = match request.runtime {
            Runtime::Qairt => "qairt",
            Runtime::LlamaCpp => "llama_cpp",
            Runtime::Unknown => {
                return Err(Error::InvalidInput(
                    "this model's runtime can't be benchmarked".into(),
                ));
            }
        };
        let runs = self.config.tools_dir.join("runs");
        tokio::fs::create_dir_all(&runs).await?;
        let report_path = runs.join(format!("{}-{}.json", ctx.id(), device(unit)));
        let _ = tokio::fs::remove_file(&report_path).await;

        let mut command = Command::new(&binary);
        command
            .args(["--plugin", plugin, "--device", device(unit), "-m"])
            .arg(&request.model)
            .args(["--warmup", "1", "-r"])
            .arg(request.repetitions.to_string())
            .arg("-p")
            .arg(request.prompt_tokens.to_string())
            .arg("-n")
            .arg(request.generated_tokens.to_string())
            .arg("--power-mode")
            .arg(&request.power_mode)
            .arg("--output-json")
            .arg(&report_path)
            .args(["--cell-id", "calcine"]);
        if let Some(data_dir) = &self.geniex.config().data_dir {
            command.arg("--mm-data-dir").arg(data_dir);
        }
        command
            .env("GENIEX_LOG", "error")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        tracing::info!(model = %request.model, ?unit, "starting geniex-bench");

        let child = command.spawn()?;
        let output = tokio::select! {
            output = tokio::time::timeout(RUN_TIMEOUT, child.wait_with_output()) => output
                .map_err(|_| Error::Timeout {
                    command: "geniex-bench".into(),
                    seconds: RUN_TIMEOUT.as_secs(),
                })??,
            // Dropping the child kills it.
            () = ctx.cancelled() => return Err(Error::Cancelled),
        };
        let report = tokio::fs::read_to_string(&report_path).await;
        let _ = tokio::fs::remove_file(&report_path).await;
        match report {
            Ok(json) if output.status.success() => report::parse(&json),
            _ => {
                let text = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(Error::Command {
                    command: "geniex-bench".into(),
                    message: report::failure(&text)
                        .unwrap_or_else(|| format!("stopped without results ({})", output.status)),
                })
            }
        }
    }

    fn history(&self) -> Vec<BenchResult> {
        self.lock().clone()
    }

    fn record(&self, results: Vec<BenchResult>) -> Result<()> {
        let mut history = self.lock();
        let merged = merge_history(std::mem::take(&mut *history), results);
        *history = merged;
        self.save(&history)
    }

    fn forget(&self, ids: &[String]) -> Result<()> {
        let mut history = self.lock();
        history.retain(|result| !ids.contains(&result.id));
        self.save(&history)
    }
}

/// `geniex-bench --device` for a compute unit.
fn device(unit: ComputeUnit) -> &'static str {
    match unit {
        ComputeUnit::Npu => "npu",
        ComputeUnit::Gpu => "gpu",
        ComputeUnit::Cpu => "cpu",
        ComputeUnit::Hybrid => "hybrid",
    }
}

fn load_history(path: &Path) -> Vec<BenchResult> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Unpack with Windows' own `tar` (bsdtar), which reads zip archives and
/// refuses absolute paths and `..` entries.
async fn extract(archive: &Path, destination: &Path) -> Result<()> {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let tar = PathBuf::from(system_root).join("System32").join("tar.exe");
    let mut command = Command::new(tar);
    command
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(destination)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command.output().await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::Command {
            command: "tar".into(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }
}
