//! Documents attached in the chat.
//!
//! A file's text is read in a child process (this same executable, started
//! with [`CHILD_ARG`]): PDF readers can panic on unusual files, and release
//! builds abort on panics, which would close Calcine. When the detailed PDF
//! reader fails, a plainer one gets a second try. The text is then kept as
//! passages (see `calcine-docs`), and token counts are cached per model.
//!
//! - `child`: the child process that reads a file
//! - `import`: attaching a file, with a second try for PDFs
//! - `context`: what the model reads for a message, with token counts

mod child;
mod context;
mod import;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use calcine_docs::DocumentStore;
pub use child::run_child_if_asked;
pub use import::Imported;

/// Tokens of each passage, by model and document id.
type TokenCache = HashMap<(String, String), Arc<Vec<u32>>>;

/// The documents folder and cached token counts, as Tauri state.
#[derive(Debug)]
pub struct Documents {
    store: DocumentStore,
    counts: Mutex<TokenCache>,
}

impl Documents {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            store: DocumentStore::new(dir),
            counts: Mutex::new(HashMap::new()),
        }
    }

    /// Forget documents: their text and their token counts.
    pub async fn remove(&self, ids: Vec<String>) {
        self.counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(_, id), _| !ids.contains(id));
        let store = self.store.clone();
        let _ = tokio::task::spawn_blocking(move || store.remove(&ids)).await;
    }

    /// Delete the documents no conversation uses any more.
    pub async fn retain(&self, keep: Vec<String>) -> u32 {
        let store = self.store.clone();
        let removed = tokio::task::spawn_blocking(move || store.retain(&keep))
            .await
            .unwrap_or_default();
        u32::try_from(removed).unwrap_or(u32::MAX)
    }
}
