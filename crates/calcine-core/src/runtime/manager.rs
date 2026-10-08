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

    /// Pin the chipset models are downloaded for (`geniex config set chipset`),
    /// or go back to detecting it with `None`.
    async fn set_chipset(&self, chipset: Option<&str>) -> Result<()>;

    /// Where models are cached on disk.
    fn models_dir(&self) -> Option<PathBuf>;
}
