//! Service traits implemented by each backend.
//!
//! Each trait covers one domain so backends can be swapped piecemeal (the CLI
//! adapter today, an FFI adapter on the GenieX C SDK later).

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::Result;
use crate::hardware::HardwareInfo;
use crate::jobs::JobCtx;
use crate::model::{HubCatalog, LocalModel, ModelKey};
use crate::reference::PullRequest;
use crate::runtime::RuntimeInfo;

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

/// The GenieX runtime installation itself.
#[async_trait]
pub trait RuntimeManager: Send + Sync {
    /// Locate GenieX and read its versions. Fails with
    /// [`Error::RuntimeNotFound`](crate::Error::RuntimeNotFound) when absent.
    async fn info(&self) -> Result<RuntimeInfo>;

    /// The configured or detected Snapdragon chipset (`geniex config get chipset`).
    async fn chipset(&self) -> Result<Option<String>>;

    /// Where models are cached on disk.
    fn models_dir(&self) -> Option<PathBuf>;
}

/// The device's compute units, memory and storage.
#[async_trait]
pub trait HardwareProbe: Send + Sync {
    /// A one-off description of the hardware. `models_dir` selects which drive
    /// to report free space for.
    async fn snapshot(&self, models_dir: Option<&Path>) -> Result<HardwareInfo>;
}
