//! Models: what's installed, what can be downloaded, and how to refer to them.

mod reference;
mod store;
mod types;

pub use reference::{ModelHub, ModelReference, PullRequest};
pub use store::{ModelCatalog, ModelStore};
pub use types::{ComputeUnit, HubCatalog, HubModel, LocalModel, ModelKey, ModelType, Runtime};
