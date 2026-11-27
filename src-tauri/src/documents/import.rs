//! Attaching a file: its text read, then kept as passages.

use std::path::Path;

use calcine_docs::{DocumentInfo, DocumentKind, DocumentProblem, MAX_FILE_BYTES, PdfReader};
use serde::Serialize;
use specta::Type;

use super::Documents;
use super::child::{ChildOutput, read_in_child};

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

impl Documents {
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
}
