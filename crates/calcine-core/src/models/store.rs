//! Service traits for models.

use async_trait::async_trait;

use super::reference::PullRequest;
use super::types::{HubCatalog, LocalModel, ModelKey};
use crate::Result;
use crate::jobs::JobCtx;

/// Models in the local GenieX cache (`list`, `pull`, `remove`, …).
#[async_trait]
pub trait ModelStore: Send + Sync {
    /// Every cached model (`geniex list`).
    async fn list(&self) -> Result<Vec<LocalModel>>;

    /// Download a model (`geniex pull`), reporting progress through `ctx` and
    /// returning [`Error::Cancelled`](crate::Error::Cancelled) when cancelled.
    async fn pull(&self, request: PullRequest, ctx: JobCtx) -> Result<()>;

    /// Delete models or single precisions (`geniex remove`).
    async fn remove(&self, keys: &[ModelKey]) -> Result<()>;
}

/// Models available to download.
#[async_trait]
pub trait ModelCatalog: Send + Sync {
    /// Qualcomm AI Hub models with an NPU build (`geniex model list`).
    /// `all_chipsets` lists every model instead of only this device's.
    async fn aihub(&self, all_chipsets: bool) -> Result<HubCatalog>;
}
