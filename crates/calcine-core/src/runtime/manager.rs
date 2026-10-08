use std::path::PathBuf;

use async_trait::async_trait;

use super::types::RuntimeInfo;
use crate::Result;

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
