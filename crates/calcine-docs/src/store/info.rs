//! What the chat knows about a document, and what's saved of it.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::kind::DocumentKind;
use crate::passages::Passage;

/// What the chat knows about an attached document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    pub id: String,
    /// File name, as shown to the user and the model.
    pub name: String,
    pub kind: DocumentKind,
    /// Pages or slides, when the format has them.
    pub pages: Option<u32>,
    /// Characters of text found.
    pub chars: u32,
    /// Size of the original file.
    pub bytes: u64,
    pub passages: u32,
}

/// A document's passages, as saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredDocument {
    pub info: DocumentInfo,
    pub passages: Vec<Passage>,
}
