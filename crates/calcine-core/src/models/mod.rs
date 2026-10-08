//! Models: what's installed, what can be downloaded, and how to refer to them.

mod reference;
mod store;
mod types;

pub use reference::{ModelHub, ModelReference, PullRequest};
pub use store::{ModelCatalog, ModelDirectory, ModelStore};
pub use types::{
    Chipset, ComputeUnit, HubCatalog, HubModel, LocalModel, ModelKey, ModelType, RemoteModel,
    RemoteModelDetails, RemotePrecision, Runtime,
};
