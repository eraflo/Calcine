//! Tests that link Tauri live here, not in `src/`: on Windows only integration
//! test binaries get the app manifest they need (see `build.rs`).

/// Keeps `src/lib/bindings.ts` in sync; CI fails if this leaves a git diff.
#[test]
fn export_bindings() {
    calcine_lib::export_bindings().expect("bindings should export");
}
