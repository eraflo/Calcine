//! Extracted documents on disk: one JSON file per document with its
//! passages. Only the text is kept, never the original file; it's deleted
//! with the conversation that uses it.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::extract::{DocumentKind, Extracted};
use crate::passages::{Passage, passages};
use crate::{Error, Result};

/// Longest text kept for one document, in characters (a few thousand pages).
const MAX_CHARS: usize = 4_000_000;

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

/// The documents folder.
#[derive(Debug, Clone)]
pub struct DocumentStore {
    dir: PathBuf,
}

impl DocumentStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        // Ids come from the webview: nothing but the hex ids made here.
        if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::NotFound);
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    /// Cut `extracted` into passages and save it under a new id.
    pub fn add(&self, name: &str, bytes: u64, extracted: &Extracted) -> Result<DocumentInfo> {
        let chars: usize = extracted
            .pages
            .iter()
            .map(|page| page.chars().count())
            .sum();
        if chars > MAX_CHARS {
            return Err(Error::TooLarge(format!(
                "{} million characters",
                chars / 1_000_000
            )));
        }
        let passages = passages(extracted);
        if passages.is_empty() {
            return Err(Error::NoText);
        }
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).map_err(|err| Error::Io(std::io::Error::other(err)))?;
        let info = DocumentInfo {
            id: hex::encode(random),
            name: name.to_owned(),
            kind: extracted.kind,
            pages: extracted
                .paged
                .then(|| u32::try_from(extracted.pages.len()).unwrap_or(u32::MAX)),
            chars: u32::try_from(chars).unwrap_or(u32::MAX),
            bytes,
            passages: u32::try_from(passages.len()).unwrap_or(u32::MAX),
        };
        let stored = StoredDocument {
            info: info.clone(),
            passages,
        };
        std::fs::create_dir_all(&self.dir)?;
        let path = self.path(&info.id)?;
        let partial = path.with_extension("json.partial");
        let json = serde_json::to_vec(&stored).map_err(std::io::Error::other)?;
        std::fs::write(&partial, json)?;
        std::fs::rename(&partial, &path)?;
        Ok(info)
    }

    pub fn get(&self, id: &str) -> Result<StoredDocument> {
        let path = self.path(id)?;
        let json = match std::fs::read(&path) {
            Ok(json) => json,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::NotFound);
            }
            Err(err) => return Err(err.into()),
        };
        serde_json::from_slice(&json).map_err(|err| Error::Read(err.to_string()))
    }

    /// Delete these documents; unknown ids are ignored.
    pub fn remove(&self, ids: &[String]) {
        for id in ids {
            if let Ok(path) = self.path(id)
                && let Err(err) = std::fs::remove_file(&path)
                && err.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(%err, path = %path.display(), "couldn't delete a document");
            }
        }
    }

    /// Delete every document but `keep`: those of deleted conversations.
    /// Returns how many were deleted.
    pub fn retain(&self, keep: &[String]) -> usize {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return 0;
        };
        let mut removed = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let id = name
                .strip_suffix(".json.partial")
                .or_else(|| name.strip_suffix(".json"));
            if let Some(id) = id
                && (name.ends_with(".partial") || !keep.iter().any(|kept| kept == id))
                && std::fs::remove_file(&path).is_ok()
            {
                removed += 1;
            }
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut random = [0u8; 8];
        getrandom::fill(&mut random).unwrap();
        std::env::temp_dir().join(format!("calcine-docs-{name}-{}", hex::encode(random)))
    }

    fn sample() -> Extracted {
        Extracted {
            kind: DocumentKind::Pdf,
            pages: vec!["First page.".into(), "Second page.".into()],
            paged: true,
        }
    }

    #[test]
    fn documents_round_trip() {
        let dir = temp_dir("round-trip");
        let store = DocumentStore::new(&dir);
        let info = store.add("report.pdf", 1234, &sample()).unwrap();
        assert_eq!(info.pages, Some(2));
        assert_eq!(info.passages, 2);
        assert_eq!(info.chars, 23);
        let stored = store.get(&info.id).unwrap();
        assert_eq!(stored.info, info);
        assert_eq!(stored.passages[1].page, Some(2));
        store.remove(std::slice::from_ref(&info.id));
        assert!(matches!(store.get(&info.id), Err(Error::NotFound)));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn ids_cant_leave_the_folder() {
        let store = DocumentStore::new(temp_dir("ids"));
        assert!(matches!(store.get("../secrets"), Err(Error::NotFound)));
        assert!(matches!(store.get(&"a".repeat(31)), Err(Error::NotFound)));
    }

    #[test]
    fn empty_documents_are_refused() {
        let store = DocumentStore::new(temp_dir("empty"));
        let empty = Extracted {
            kind: DocumentKind::Pdf,
            pages: vec!["  \n ".into()],
            paged: true,
        };
        assert!(matches!(
            store.add("scan.pdf", 1, &empty),
            Err(Error::NoText)
        ));
    }

    #[test]
    fn retain_deletes_the_others() {
        let dir = temp_dir("retain");
        let store = DocumentStore::new(&dir);
        let kept = store.add("a.pdf", 1, &sample()).unwrap();
        let dropped = store.add("b.pdf", 1, &sample()).unwrap();
        std::fs::write(dir.join("stray.txt"), "not ours").unwrap();
        assert_eq!(store.retain(std::slice::from_ref(&kept.id)), 1);
        assert!(store.get(&kept.id).is_ok());
        assert!(matches!(store.get(&dropped.id), Err(Error::NotFound)));
        assert!(dir.join("stray.txt").exists(), "other files are left alone");
        std::fs::remove_dir_all(dir).ok();
    }
}
