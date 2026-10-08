//! The GenieX runtime: versions, configuration, data location, and the
//! inference server process.

mod manager;
mod server;
mod types;

pub use manager::RuntimeManager;
pub use server::{InferenceServer, ServerState};
pub use types::RuntimeInfo;
