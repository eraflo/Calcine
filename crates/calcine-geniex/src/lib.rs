//! GenieX CLI adapter.
//!
//! Implements the `calcine-core` service traits by spawning the official
//! `geniex` executable and parsing its output. Parsers live in [`parse`] and
//! are tested against outputs captured from a real installation.

pub mod discovery;
pub mod parse;
pub mod runner;

use std::path::PathBuf;

use async_trait::async_trait;
use calcine_core::model::LocalModel;
use calcine_core::runtime::RuntimeInfo;
use calcine_core::traits::{ModelStore, RuntimeManager};
use calcine_core::{Error, Result};

use crate::runner::Cli;

/// Options for locating and invoking GenieX.
#[derive(Debug, Clone, Default)]
pub struct GeniexConfig {
    /// Use this executable instead of auto-discovery.
    pub binary: Option<PathBuf>,
    /// Custom model cache (`--data-dir`).
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
}
