//! GenieX CLI adapter.
//!
//! Implements the `calcine-core` service traits by spawning the official
//! `geniex` executable and parsing its output. Parsers live in [`parse`] and
//! are tested against outputs captured from a real GenieX install.

pub mod discovery;
pub mod parse;
mod pull;
pub mod runner;

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use calcine_core::jobs::JobCtx;
use calcine_core::model::{HubCatalog, LocalModel, ModelKey};
use calcine_core::reference::PullRequest;
use calcine_core::runtime::RuntimeInfo;
use calcine_core::traits::{ModelCatalog, ModelStore, RuntimeManager};
use calcine_core::{Error, Result};

use crate::runner::Cli;

/// `geniex model list` queries Qualcomm AI Hub over the network.
const CATALOG_TIMEOUT: Duration = Duration::from_secs(90);

/// Options for locating and invoking GenieX.
#[derive(Debug, Clone, Default)]
pub struct GeniexConfig {
    /// Use this executable instead of auto-discovery.
    pub binary: Option<PathBuf>,
    /// Custom data directory (`--data-dir`).
    pub data_dir: Option<PathBuf>,
}

/// Backend backed by the GenieX CLI.
#[derive(Debug, Clone, Default)]
pub struct Geniex {
    config: GeniexConfig,
}

impl Geniex {
    pub fn new(config: GeniexConfig) -> Self {
        Self { config }
    }

    /// Resolve the executable on every call so installs, updates and
    /// uninstalls are picked up without restarting Calcine.
    fn cli(&self) -> Result<Cli> {
        let binary =
            discovery::locate(self.config.binary.as_deref()).ok_or(Error::RuntimeNotFound)?;
        Ok(Cli::new(binary, self.config.data_dir.clone()))
    }
}

#[async_trait]
impl ModelStore for Geniex {
    async fn list(&self) -> Result<Vec<LocalModel>> {
        let output = self.cli()?.run(&["list", "--format", "json"]).await?;
        parse::list_json::parse(&output)
    }

    async fn pull(&self, request: PullRequest, ctx: JobCtx) -> Result<()> {
        pull::run(&self.cli()?, &request, &ctx).await
    }

    async fn remove(&self, keys: &[ModelKey]) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        let keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
        let mut args = vec!["remove", "--yes"];
        args.extend(keys.iter().map(String::as_str));
        self.cli()?.run(&args).await.map(drop)
    }
}

#[async_trait]
impl ModelCatalog for Geniex {
    async fn aihub(&self, all_chipsets: bool) -> Result<HubCatalog> {
        let cli = self.cli()?.with_timeout(CATALOG_TIMEOUT);
        let args: &[&str] = if all_chipsets {
            &["model", "list", "--all"]
        } else {
            &["model", "list"]
        };
        parse::hub_table::parse(&cli.run(args).await?)
    }
}

#[async_trait]
impl RuntimeManager for Geniex {
    async fn info(&self) -> Result<RuntimeInfo> {
        let cli = self.cli()?;
        let output = cli.run(&["version"]).await?;
        let versions = parse::version::parse(&output)?;
        Ok(RuntimeInfo {
            cli_version: versions.cli,
            qairt_version: versions.qairt,
            llama_cpp_hash: versions.llama_cpp,
            binary_path: cli.binary().display().to_string(),
        })
    }

    async fn chipset(&self) -> Result<Option<String>> {
        let output = self.cli()?.run(&["config", "get", "chipset"]).await?;
        let chipset = output.trim();
        Ok(
            (!chipset.is_empty() && !chipset.eq_ignore_ascii_case("unknown"))
                .then(|| chipset.to_owned()),
        )
    }

    fn models_dir(&self) -> Option<PathBuf> {
        discovery::data_dir(self.config.data_dir.as_deref()).map(|dir| dir.join("models"))
    }
}
