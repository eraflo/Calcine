//! The GenieX runtime: versions, configuration, data location, the
//! inference server process, and installing or updating GenieX itself.

mod manager;
mod server;
mod types;
mod update;

pub use manager::RuntimeManager;
pub use server::{InferenceServer, ServerState};
pub use types::RuntimeInfo;
pub use update::{
    CachedInstaller, InstallSource, InstallerAsset, ReleaseChannel, RuntimeInstaller,
    RuntimeRelease, RuntimeUpdateCheck, compare_versions,
};
