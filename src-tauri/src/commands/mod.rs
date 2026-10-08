//! Tauri commands, one module per domain.
//!
//! To add a command: write it here with `#[tauri::command]` and
//! `#[specta::specta]`, register it in `bindings::builder`, then run
//! `cargo test -p calcine` to regenerate `src/lib/bindings.ts`.

// Tauri injects `State` by value.
#![allow(clippy::needless_pass_by_value)]

pub mod app;
pub mod jobs;
pub mod models;
pub mod runtime;
