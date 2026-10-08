//! Calcine domain layer.
//!
//! Pure types, service traits, errors and job orchestration shared by every
//! backend (GenieX CLI, mock) and every front door (Tauri commands, HTTP
//! gateway). This crate must not depend on Tauri or any HTTP framework so it
//! stays testable in isolation.

pub mod error;
pub mod hardware;
pub mod jobs;
pub mod model;
pub mod reference;
pub mod runtime;
pub mod services;
pub mod traits;

pub use error::{Error, ErrorKind, Result};
pub use services::{BackendKind, Services};
