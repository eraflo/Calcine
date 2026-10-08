use serde::{Deserialize, Serialize};
use specta::Type;

/// The installed GenieX runtime, as reported by `geniex version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    /// GenieX CLI version, e.g. `v0.8.0`.
    pub cli_version: String,
    /// Bundled Qualcomm AI Engine Direct (QAIRT) version, e.g. `2.45`.
    pub qairt_version: Option<String>,
    /// Bundled llama.cpp build hash.
    pub llama_cpp_hash: Option<String>,
    /// Absolute path of the `geniex` executable.
    pub binary_path: String,
}
