//! Simulated Hugging Face and AI Hub lookups.

use async_trait::async_trait;
use calcine_core::models::{
    Chipset, ModelDirectory, ModelHub, ModelReference, RemoteModel, RemoteModelDetails,
};
use calcine_core::{Error, Result};

use crate::{MockBackend, data};

#[async_trait]
impl ModelDirectory for MockBackend {
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<RemoteModel>> {
        let query = query.trim().to_ascii_lowercase();
        let mut models: Vec<RemoteModel> = data::hub_models()
            .into_iter()
            .filter(|model| model.name.to_ascii_lowercase().contains(&query))
            .collect();
        models.sort_by_key(|model| std::cmp::Reverse(model.downloads));
        models.truncate(limit as usize);
        Ok(models)
    }

    async fn details(&self, reference: &ModelReference) -> Result<RemoteModelDetails> {
        if reference.name.contains("missing") {
            return Err(Error::Network(
                "Hugging Face doesn't have this model, or it's private".into(),
            ));
        }
        match reference.hub {
            ModelHub::HuggingFace => Ok(data::hub_details(&reference.name)),
            ModelHub::Auto if !reference.name.starts_with("qualcomm/") => {
                Ok(data::hub_details(&reference.name))
            }
            _ => Err(Error::NotImplemented("Listing precisions for this hub")),
        }
    }

    async fn chipsets(&self) -> Result<Vec<Chipset>> {
        Ok(data::chipsets())
    }
}
