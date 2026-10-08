//! Service traits implemented by each backend.
//!
//! Each trait covers one domain so backends can be swapped piecemeal (the CLI
//! adapter today, an FFI adapter on the GenieX C SDK later).

use async_trait::async_trait;

use crate::Result;
use crate::model::LocalModel;
use crate::runtime::RuntimeInfo;

/// Models in the local GenieX cache (`list`, `pull`, `remove`, …).
#[async_trait]
pub trait ModelStore: Send + Sync {
    /// Every cached model (`geniex list`).
    async fn list(&self) -> Result<Vec<LocalModel>>;
}

/// The GenieX runtime installation itself.
#[async_trait]
pub trait RuntimeManager: Send + Sync {
    /// Locate GenieX and read its versions. Fails with
    /// [`Error::RuntimeNotFound`](crate::Error::RuntimeNotFound) when absent.
    async fn info(&self) -> Result<RuntimeInfo>;
}
