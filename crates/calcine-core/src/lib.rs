//! Calcine domain layer.
//!
//! Pure types, service traits, errors and job orchestration shared by every
//! backend (GenieX CLI, mock) and every front door (Tauri commands, HTTP
//! gateway). This crate must not depend on Tauri or any HTTP framework so it
//! stays testable in isolation.
//!
//! Each domain folder holds its data types (`types.rs`) and the trait a
//! backend implements for it:
//!
//! | Module | Types | Service trait |
//! |---|---|---|
//! | [`models`] | `LocalModel`, `HubModel`, `ModelReference`, `RemoteModel`… | `ModelStore`, `ModelCatalog`, `ModelDirectory` |
//! | [`runtime`] | `RuntimeInfo` | `RuntimeManager` |
//! | [`hardware`] | `HardwareInfo`, `HardwareUsage`… | `HardwareProbe` |
//! | [`jobs`] | `Job`, `JobState`… | — (`JobManager` runs them) |
//! | [`bench`] | `BenchRequest`, `BenchResult`… | `Benchmarker` |
//! | [`context`] | `TokenCount`, `ContextUsage` | `ContextMeter` |

pub mod bench;
pub mod context;
pub mod error;
pub mod hardware;
pub mod jobs;
pub mod models;
pub mod runtime;
pub mod services;

pub use error::{Error, ErrorKind, Result};
pub use services::{BackendKind, Services};
