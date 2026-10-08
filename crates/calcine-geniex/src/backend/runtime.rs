//! `geniex version` and `config get`.

use std::path::PathBuf;

use async_trait::async_trait;
use calcine_core::Result;
use calcine_core::runtime::{RuntimeInfo, RuntimeManager};

use crate::cli::discovery;
use crate::{Geniex, parse};

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
        discovery::data_dir(self.config().data_dir.as_deref()).map(|dir| dir.join("models"))
    }
}
