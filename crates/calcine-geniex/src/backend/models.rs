//! `geniex list`, `pull`, `remove`, `clean`, `model set-type` and `model list`.

use std::time::Duration;

use async_trait::async_trait;
use calcine_core::Error;
use calcine_core::Result;
use calcine_core::jobs::JobCtx;
use calcine_core::models::{
    HubCatalog, LocalModel, ModelCatalog, ModelKey, ModelStore, ModelType, PullRequest,
};

use super::pull;
use crate::{Geniex, parse};

/// `geniex model list` queries Qualcomm AI Hub over the network.
const CATALOG_TIMEOUT: Duration = Duration::from_secs(90);

/// Deleting tens of gigabytes can take a while on a slow drive.
const CLEAN_TIMEOUT: Duration = Duration::from_secs(300);

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

    async fn clean(&self) -> Result<()> {
        self.cli()?
            .with_timeout(CLEAN_TIMEOUT)
            .run(&["clean", "--yes"])
            .await
            .map(drop)
    }

    async fn set_type(&self, name: &str, model_type: ModelType) -> Result<()> {
        let value = match model_type {
            ModelType::Llm => "llm",
            ModelType::Vlm => "vlm",
            ModelType::Unknown => {
                return Err(Error::InvalidInput("choose LLM or VLM".into()));
            }
        };
        // Always pass the type: without it GenieX opens an interactive picker.
        self.cli()?
            .run(&["model", "set-type", name, value])
            .await
            .map(drop)
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
