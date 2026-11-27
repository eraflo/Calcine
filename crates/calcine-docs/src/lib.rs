//! Documents attached in Calcine's chat.
//!
//! Small models have small context windows (4096 tokens for most NPU
//! builds), so a document is rarely read whole. Calcine keeps its text cut
//! into passages and, for each message, shows the model the passages that
//! match it best, with their page, within the room the conversation leaves.
//!
//! - `kind`: what a file is, from its extension
//! - `extract`: the text of PDF, Office, OpenDocument, HTML and text files
//! - `passages`: cleaned text cut into passages, each with its page
//! - `search`: keyword ranking (BM25), no embedding model needed
//! - `select`: what the model is shown within a token budget
//! - `store`: extracted documents kept on disk, text only
//! - `error`: why a file couldn't be read

mod error;
mod extract;
mod kind;
mod passages;
mod search;
mod select;
mod store;

pub use error::{DocumentProblem, Error, Result};
pub use extract::{Extracted, PdfReader, extract, extract_with};
pub use kind::{DocumentKind, SUPPORTED_EXTENSIONS, kind_of};
pub use passages::{Passage, passages};
pub use search::{Index, terms};
pub use select::{Candidate, ContextMode, DocumentContext, Source, select};
pub use store::{DocumentInfo, DocumentStore, StoredDocument};

/// Largest file Calcine reads.
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
