//! Tauri commands, one module per domain. Keep them thin: logic belongs in
//! the crates.

// Tauri injects `State` by value.
#![allow(clippy::needless_pass_by_value)]

pub mod app;
pub mod bench;
pub mod jobs;
pub mod models;
pub mod runtime;
pub mod server;
pub mod updates;
