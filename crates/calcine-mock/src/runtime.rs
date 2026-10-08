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
        Ok(Some(
            self.pinned_chipset()
                .clone()
                .unwrap_or_else(|| data::CHIPSET.into()),
        ))
    }

    async fn set_chipset(&self, chipset: Option<&str>) -> Result<()> {
        *self.pinned_chipset() = chipset
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        Ok(())
    }

    fn models_dir(&self) -> Option<PathBuf> {
        Some(PathBuf::from("mock://models"))
    }
}
