use std::path::PathBuf;

use async_trait::async_trait;
use calcine_core::Result;
use calcine_core::runtime::{RuntimeInfo, RuntimeManager};

use crate::{MockBackend, data};

#[async_trait]
impl RuntimeManager for MockBackend {
    async fn info(&self) -> Result<RuntimeInfo> {
        Ok(data::runtime_info())
    }

    async fn chipset(&self) -> Result<Option<String>> {
        Ok(Some(data::CHIPSET.into()))
    }

    fn models_dir(&self) -> Option<PathBuf> {
        Some(PathBuf::from("mock://models"))
    }
}
