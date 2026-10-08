//! Calcine domain layer.
//!
//! Pure types, service traits and errors shared by every backend (GenieX CLI,
//! mock) and every front door (Tauri commands, HTTP gateway). This crate must
//! not depend on Tauri or any HTTP framework so it stays testable in isolation.

pub mod error;
pub mod model;
pub mod runtime;
pub mod services;
pub mod traits;

pub use error::{Error, ErrorKind, Result};
pub use services::{BackendKind, Services};
