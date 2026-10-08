//! Parsers for GenieX CLI output.
//!
//! Each parser is tested against output captured from a real GenieX install
//! (`tests/fixtures/`). Prefer machine-readable formats (`--format json`) and
//! keep text parsers tolerant: unknown lines are skipped, never fatal.

pub mod config_list;
pub mod hub_table;
pub mod list_json;
pub mod progress;
pub mod version;

use calcine_core::model::{ModelType, Runtime};

/// Map a CLI runtime id (`qairt`, `llama_cpp`) to [`Runtime`].
pub(crate) fn runtime(value: &str) -> Runtime {
    match value.trim().to_ascii_lowercase().as_str() {
        "qairt" => Runtime::Qairt,
        "llama_cpp" | "llamacpp" | "llama.cpp" => Runtime::LlamaCpp,
        _ => Runtime::Unknown,
    }
}

/// Map a CLI model type (`llm`, `vlm`) to [`ModelType`].
pub(crate) fn model_type(value: &str) -> ModelType {
    match value.trim().to_ascii_lowercase().as_str() {
        "llm" => ModelType::Llm,
        "vlm" => ModelType::Vlm,
        _ => ModelType::Unknown,
    }
}

#[cfg(test)]
pub(crate) fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}
