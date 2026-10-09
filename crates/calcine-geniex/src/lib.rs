//! GenieX CLI adapter.
//!
//! Implements the `calcine-core` service traits by spawning the official
//! `geniex` executable and parsing its output.
//!
//! - [`cli`]: finding the executable and running commands
//! - `backend`: the service trait implementations (models, pull, runtime)
//! - [`parse`]: output parsers, tested against captures from a real install
//! - [`update`]: installing and updating GenieX with its official installer
//! - [`bench`]: benchmarks with Qualcomm's `geniex-bench`
//! - `context`: context windows and token counts

mod backend;
pub use backend::GeniexServer;
pub mod bench;
pub use bench::{BenchConfig, GeniexBench};
pub mod cli;
mod context;
pub use context::GeniexContext;
pub mod parse;
mod platform;
pub mod update;
pub use update::{BundledInstaller, GeniexInstaller, InstallerConfig};

use std::path::PathBuf;

use calcine_core::{Error, Result};

use crate::cli::Cli;

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
    pub(crate) fn cli(&self) -> Result<Cli> {
        let binary =
            cli::discovery::locate(self.config.binary.as_deref()).ok_or(Error::RuntimeNotFound)?;
        Ok(Cli::new(binary, self.config.data_dir.clone()))
    }

    pub(crate) fn config(&self) -> &GeniexConfig {
        &self.config
    }
}
