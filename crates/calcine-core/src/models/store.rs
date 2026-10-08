//! Service traits for models.

use async_trait::async_trait;

use super::reference::ModelReference;
use super::reference::PullRequest;
use super::types::{
    Chipset, HubCatalog, LocalModel, ModelKey, ModelType, RemoteModel, RemoteModelDetails,
};
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

    /// Delete every cached model (`geniex clean`).
    async fn clean(&self) -> Result<()>;

    /// Correct whether a cached model takes images and audio (`geniex model set-type`).
    async fn set_type(&self, name: &str, model_type: ModelType) -> Result<()>;
}

/// Models available to download.
#[async_trait]
pub trait ModelCatalog: Send + Sync {
    /// Qualcomm AI Hub models with an NPU build (`geniex model list`).
    /// `all_chipsets` lists every model instead of only this device's.
    async fn aihub(&self, all_chipsets: bool) -> Result<HubCatalog>;
}

/// Model hubs queried directly over HTTP, for what GenieX only shows in
/// interactive prompts: search, precisions with sizes, chipset names.
#[async_trait]
pub trait ModelDirectory: Send + Sync {
    /// Search Hugging Face for GGUF models, most downloaded first.
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<RemoteModel>>;

    /// Precisions available for a model, with download sizes. Supported for
    /// Hugging Face; other hubs return
    /// [`Error::NotImplemented`](crate::Error::NotImplemented).
    async fn details(&self, reference: &ModelReference) -> Result<RemoteModelDetails>;

    /// Chipsets Qualcomm AI Hub builds models for, on this OS.
    async fn chipsets(&self) -> Result<Vec<Chipset>>;
}
