//! Documents attached in the chat.
//!
//! A file's text is read in a child process (this same executable, started
//! with [`CHILD_ARG`]): PDF readers can panic on unusual files, and release
//! builds abort on panics, which would close Calcine. When the detailed PDF
//! reader fails, a plainer one gets a second try. The text is then kept as
//! passages (see `calcine-docs`), and token counts are cached per model.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use calcine_core::Services;
use calcine_docs::{
    Candidate, DocumentContext, DocumentInfo, DocumentKind, DocumentProblem, DocumentStore,
    Extracted, MAX_FILE_BYTES, PdfReader, StoredDocument,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;

/// First argument of the child process that reads a file:
/// `calcine --extract-document <detailed|simple> <file> <output.json>`.
const CHILD_ARG: &str = "--extract-document";
/// A file that takes longer than this to read is given up on.
const READ_TIMEOUT: Duration = Duration::from_secs(120);
/// How much each of the user's last messages counts when ranking passages,
/// newest first.
const QUERY_WEIGHTS: [f32; 3] = [1.0, 0.5, 0.25];

/// What the child process writes.
#[derive(Debug, Serialize, Deserialize)]
enum ChildOutput {
    Text(Extracted),
    Failed {
        problem: DocumentProblem,
        detail: String,
    },
}

/// When this process was started to read a document: read it and return
/// the exit code. `None` for a normal start.
pub fn run_child_if_asked() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some(CHILD_ARG) {
        return None;
    }
    let (Some(reader), Some(input), Some(output)) = (args.get(2), args.get(3), args.get(4)) else {
        return Some(2);
    };
    let reader = if reader == "simple" {
        PdfReader::Simple
    } else {
        PdfReader::Detailed
    };
    let result = match calcine_docs::extract_with(Path::new(input), reader) {
        Ok(text) => ChildOutput::Text(text),
        Err(err) => ChildOutput::Failed {
            problem: err.problem(),
            detail: err.to_string(),
        },
    };
    let Ok(json) = serde_json::to_vec(&result) else {
        return Some(3);
    };
    Some(if std::fs::write(output, json).is_ok() {
        0
    } else {
        4
    })
}

/// The outcome of attaching a file.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Imported {
    Ready {
        document: DocumentInfo,
    },
    Failed {
        name: String,
        problem: DocumentProblem,
        detail: String,
    },
}

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

    /// Read a file and keep its text.
    pub async fn import(&self, path: &Path) -> Imported {
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        let failed = |problem, detail: String| Imported::Failed {
            name: name.clone(),
            problem,
            detail,
        };
        let Some(kind) = calcine_docs::kind_of(path) else {
            return failed(DocumentProblem::Unsupported, String::new());
        };
        let bytes = match tokio::fs::metadata(path).await {
            Ok(metadata) if metadata.is_file() => metadata.len(),
            Ok(_) => return failed(DocumentProblem::Unreadable, "not a file".to_owned()),
            Err(err) => return failed(DocumentProblem::Unreadable, err.to_string()),
        };
        if bytes > MAX_FILE_BYTES {
            return failed(
                DocumentProblem::TooLarge,
                format!("{} MB", bytes / 1024 / 1024),
            );
        }

        let mut output = read_in_child(path, PdfReader::Detailed).await;
        let retry = kind == DocumentKind::Pdf
            && match &output {
                Err(_) => true,
                Ok(ChildOutput::Failed { problem, .. }) => *problem == DocumentProblem::Unreadable,
                Ok(ChildOutput::Text(_)) => false,
            };
        if retry {
            if let Err(reason) | Ok(ChildOutput::Failed { detail: reason, .. }) = &output {
                tracing::info!(%reason, "reading the PDF again with the simple reader");
            }
            output = read_in_child(path, PdfReader::Simple).await;
        }

        match output {
            Ok(ChildOutput::Text(extracted)) => {
                let store = self.store.clone();
                let saved = {
                    let name = name.clone();
                    tokio::task::spawn_blocking(move || store.add(&name, bytes, &extracted)).await
                };
                match saved {
                    Ok(Ok(document)) => Imported::Ready { document },
                    Ok(Err(err)) => failed(err.problem(), err.to_string()),
                    Err(err) => failed(DocumentProblem::Unreadable, err.to_string()),
                }
            }
            Ok(ChildOutput::Failed { problem, detail }) => failed(problem, detail),
            Err(detail) => {
                tracing::warn!(%detail, "couldn't read a document");
                failed(DocumentProblem::Unreadable, detail)
            }
        }
    }

    /// What the model reads of `ids` for the user's last `queries` (newest
    /// first), within `budget` tokens.
    pub async fn context(
        &self,
        services: &Services,
        model: &str,
        ids: &[String],
        queries: &[String],
        budget: u32,
    ) -> calcine_core::Result<DocumentContext> {
        let store = self.store.clone();
        let wanted = ids.to_vec();
        let documents: Vec<StoredDocument> = tokio::task::spawn_blocking(move || {
            wanted
                .iter()
                .filter_map(|id| {
                    store
                        .get(id)
                        .inspect_err(|err| tracing::warn!(%err, %id, "missing document"))
                        .ok()
                })
                .collect()
        })
        .await
        .map_err(|err| calcine_core::Error::Io(std::io::Error::other(err)))?;

        let mut counts = Vec::with_capacity(documents.len());
        for document in &documents {
            counts.push(self.token_counts(services, model, document).await?);
        }
        let candidates: Vec<Candidate<'_>> = documents
            .iter()
            .zip(&counts)
            .map(|(document, tokens)| Candidate {
                document,
                tokens: tokens.as_slice(),
            })
            .collect();
        let queries: Vec<(&str, f32)> = queries
            .iter()
            .zip(QUERY_WEIGHTS)
            .map(|(query, weight)| (query.as_str(), weight))
            .collect();
        Ok(calcine_docs::select(&candidates, &queries, budget))
    }

    /// Tokens of each passage of `document` for `model`, counted once.
    async fn token_counts(
        &self,
        services: &Services,
        model: &str,
        document: &StoredDocument,
    ) -> calcine_core::Result<Arc<Vec<u32>>> {
        let key = (model.to_owned(), document.info.id.clone());
        if let Some(counts) = self
            .counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
        {
            return Ok(counts.clone());
        }
        let messages: Vec<_> = document
            .passages
            .iter()
            .map(|passage| json!({ "role": "user", "content": passage.text }))
            .collect();
        let count = services
            .context
            .count(model, &json!({ "messages": messages }))
            .await?;
        let counts = Arc::new(count.per_message);
        self.counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, counts.clone());
        Ok(counts)
    }

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

/// Run the reader in a child process; `Err` when it crashed or hung.
async fn read_in_child(path: &Path, reader: PdfReader) -> Result<ChildOutput, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let output = std::env::temp_dir().join(format!(
        "calcine-document-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut command = tokio::process::Command::new(exe);
    command
        .arg(CHILD_ARG)
        .arg(match reader {
            PdfReader::Detailed => "detailed",
            PdfReader::Simple => "simple",
        })
        .arg(path)
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let result = match tokio::time::timeout(READ_TIMEOUT, command.status()).await {
        Err(_) => Err("reading the file took too long".to_owned()),
        Ok(Err(err)) => Err(err.to_string()),
        Ok(Ok(status)) if !status.success() => Err(format!("the reader stopped ({status})")),
        Ok(Ok(_)) => tokio::fs::read(&output)
            .await
            .map_err(|err| err.to_string())
            .and_then(|json| serde_json::from_slice(&json).map_err(|err| err.to_string())),
    };
    let _ = tokio::fs::remove_file(&output).await;
    result
}
