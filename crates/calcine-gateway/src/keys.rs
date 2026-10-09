//! API keys: one per app, shown once, stored hashed.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;

use crate::caller::Caller;
use crate::now_ms;

const TOKEN_PREFIX: &str = "calcine_";
/// Persist `last used` at most this often, to avoid a write per request.
const LAST_USED_PERSIST_MS: u64 = 60_000;

/// What a key may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum KeyScope {
    /// Chat, completions and listing models (`/v1/*`).
    Inference,
    /// Download and remove models, read jobs (`/calcine/v1/*`).
    Manage,
}

/// A key as shown in the UI (never the secret itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    /// The first characters, to recognize a key: `calcine_3f9a…`.
    pub preview: String,
    pub scopes: Vec<KeyScope>,
    /// May send local file paths and URLs (images, audio, grammars) for
    /// GenieX to read. Off by default: it lets the app read any file.
    pub allow_local_files: bool,
    /// May be used from other devices, on the local network port. Off by
    /// default: most apps run on this PC.
    #[serde(default)]
    pub network: bool,
    pub created_at_ms: u64,
    pub last_used_at_ms: Option<u64>,
}

/// What to create.
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewApiKey {
    pub name: String,
    pub scopes: Vec<KeyScope>,
    pub allow_local_files: bool,
    #[serde(default)]
    pub network: bool,
}

/// A freshly created key. `token` is only ever returned here.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreatedApiKey {
    pub token: String,
    pub key: ApiKeyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredKey {
    #[serde(flatten)]
    info: ApiKeyInfo,
    /// SHA-256 of the token, hex.
    hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("give the key a name")]
    MissingName,
    #[error("pick at least one permission")]
    NoScope,
    #[error("couldn't save API keys: {0}")]
    Io(#[from] std::io::Error),
    #[error("API keys file is corrupted: {0}")]
    Corrupted(#[from] serde_json::Error),
}

/// Keys for apps, plus Calcine's own per-launch token for its UI.
#[derive(Debug)]
pub struct KeyStore {
    path: Option<PathBuf>,
    keys: Mutex<Vec<StoredKey>>,
    /// When the file was last read or written by this store. Another process
    /// (the app and `calcine-cli`) may change it: it's read again then.
    seen: Mutex<Option<FileVersion>>,
    last_persist_ms: Mutex<u64>,
    internal_token: String,
}

impl KeyStore {
    /// Keys kept in memory only (tests, mock).
    pub fn in_memory() -> Self {
        Self::with_keys(None, Vec::new())
    }

    /// Load keys from `path` (created on first save).
    pub fn load(path: PathBuf) -> Result<Self, KeyError> {
        let keys = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(err.into()),
        };
        let store = Self::with_keys(Some(path), keys);
        *store.seen() = store.version();
        Ok(store)
    }

    fn with_keys(path: Option<PathBuf>, keys: Vec<StoredKey>) -> Self {
        Self {
            path,
            keys: Mutex::new(keys),
            seen: Mutex::new(None),
            last_persist_ms: Mutex::new(0),
            internal_token: random_token(),
        }
    }

    /// Token for Calcine's own UI. Valid until the app quits; never stored.
    pub fn internal_token(&self) -> &str {
        &self.internal_token
    }

    pub fn list(&self) -> Vec<ApiKeyInfo> {
        self.sync();
        self.lock().iter().map(|key| key.info.clone()).collect()
    }

    pub fn create(&self, request: NewApiKey) -> Result<CreatedApiKey, KeyError> {
        self.sync();
        let name = request.name.trim();
        if name.is_empty() {
            return Err(KeyError::MissingName);
        }
        if request.scopes.is_empty() {
            return Err(KeyError::NoScope);
        }
        let token = random_token();
        let mut scopes = request.scopes;
        scopes.sort_by_key(|scope| *scope as u8);
        scopes.dedup();
        let info = ApiKeyInfo {
            id: hex::encode(random_bytes::<6>()),
            name: name.to_owned(),
            preview: format!("{}…", &token[..TOKEN_PREFIX.len() + 4]),
            scopes,
            allow_local_files: request.allow_local_files,
            network: request.network,
            created_at_ms: now_ms(),
            last_used_at_ms: None,
        };
        self.lock().push(StoredKey {
            info: info.clone(),
            hash: hash(&token),
        });
        self.persist()?;
        Ok(CreatedApiKey { token, key: info })
    }

    /// Returns `false` if no key has this id.
    pub fn revoke(&self, id: &str) -> Result<bool, KeyError> {
        self.sync();
        let removed = {
            let mut keys = self.lock();
            let before = keys.len();
            keys.retain(|key| key.info.id != id);
            keys.len() != before
        };
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    /// Let a key be used from other devices, or not. `None` when there's
    /// no such key.
    pub fn set_network(&self, id: &str, network: bool) -> Result<Option<ApiKeyInfo>, KeyError> {
        self.sync();
        let changed = {
            let mut keys = self.lock();
            keys.iter_mut().find(|key| key.info.id == id).map(|key| {
                key.info.network = network;
                key.info.clone()
            })
        };
        if changed.is_some() {
            self.persist()?;
        }
        Ok(changed)
    }

    /// Who is calling with `token`, if anyone.
    pub fn authenticate(&self, token: &str) -> Option<Caller> {
        if constant_time_eq(token.as_bytes(), self.internal_token.as_bytes()) {
            return Some(Caller::Calcine);
        }
        // A key created or revoked by the other program counts at once.
        self.sync();
        let digest = hash(token);
        let now = now_ms();
        let caller = {
            let mut keys = self.lock();
            let key = keys
                .iter_mut()
                .find(|key| constant_time_eq(key.hash.as_bytes(), digest.as_bytes()))?;
            key.info.last_used_at_ms = Some(now);
            Caller::App(key.info.clone())
        };
        let due = {
            let mut last = self
                .last_persist_ms
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let due = now.saturating_sub(*last) >= LAST_USED_PERSIST_MS;
            if due {
                *last = now;
            }
            due
        };
        if due && let Err(err) = self.persist() {
            tracing::warn!(%err, "couldn't save API key usage");
        }
        Some(caller)
    }

    fn persist(&self) -> Result<(), KeyError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let json = serde_json::to_vec_pretty(&*self.lock())?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write then rename so a crash never leaves a half-written file.
        let partial = path.with_extension("json.partial");
        std::fs::write(&partial, json)?;
        std::fs::rename(partial, path)?;
        *self.seen() = self.version();
        Ok(())
    }

    fn version(&self) -> Option<FileVersion> {
        let meta = std::fs::metadata(self.path.as_ref()?).ok()?;
        Some(FileVersion {
            modified: meta.modified().ok(),
            len: meta.len(),
            inode: inode(&meta),
        })
    }

    fn seen(&self) -> MutexGuard<'_, Option<FileVersion>> {
        self.seen.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Read the file again when another program changed it.
    fn sync(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let version = self.version();
        if version == *self.seen() {
            return;
        }
        let keys = match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice::<Vec<StoredKey>>(&bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => {
                tracing::warn!(%err, "couldn't read API keys again");
                return;
            }
        };
        match keys {
            Ok(keys) => {
                *self.lock() = keys;
                *self.seen() = version;
            }
            Err(err) => {
                tracing::warn!(%err, "API keys file is unreadable, keeping the keys in memory");
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<StoredKey>> {
        self.keys.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Tells one version of the keys file from another. Linux timestamps are
/// coarse (a few milliseconds), so the size and the inode count too: each
/// save writes a new file and renames it in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileVersion {
    modified: Option<SystemTime>,
    len: u64,
    inode: u64,
}

#[cfg(unix)]
fn inode(meta: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.ino()
}

#[cfg(not(unix))]
fn inode(_meta: &std::fs::Metadata) -> u64 {
    0
}

fn random_token() -> String {
    format!("{TOKEN_PREFIX}{}", hex::encode(random_bytes::<20>()))
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes).expect("the OS random number generator is available");
    bytes
}

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_key(name: &str) -> NewApiKey {
        NewApiKey {
            name: name.into(),
            scopes: vec![KeyScope::Inference],
            allow_local_files: false,
            network: false,
        }
    }

    #[test]
    fn created_keys_authenticate_and_revoked_ones_dont() {
        let store = KeyStore::in_memory();
        let created = store.create(new_key("Continue")).unwrap();
        assert!(created.token.starts_with("calcine_"));
        assert!(created.key.preview.ends_with('…'));

        let Some(Caller::App(info)) = store.authenticate(&created.token) else {
            panic!("the key should authenticate");
        };
        assert_eq!(info.name, "Continue");
        assert!(store.list()[0].last_used_at_ms.is_some());

        assert!(store.revoke(&created.key.id).unwrap());
        assert!(store.authenticate(&created.token).is_none());
    }

    #[test]
    fn wrong_tokens_are_rejected() {
        let store = KeyStore::in_memory();
        store.create(new_key("App")).unwrap();
        assert!(store.authenticate("calcine_nope").is_none());
        assert!(store.authenticate("").is_none());
    }

    #[test]
    fn the_internal_token_is_calcine() {
        let store = KeyStore::in_memory();
        assert!(matches!(
            store.authenticate(store.internal_token()),
            Some(Caller::Calcine)
        ));
    }

    #[test]
    fn validates_new_keys() {
        let store = KeyStore::in_memory();
        assert!(matches!(
            store.create(new_key("  ")),
            Err(KeyError::MissingName)
        ));
        let no_scope = NewApiKey {
            scopes: vec![],
            ..new_key("App")
        };
        assert!(matches!(store.create(no_scope), Err(KeyError::NoScope)));
    }

    #[test]
    fn sees_keys_changed_by_another_program() {
        let dir =
            std::env::temp_dir().join(format!("calcine-keys-{}", hex::encode(random_bytes::<4>())));
        let path = dir.join("api-keys.json");
        let app = KeyStore::load(path.clone()).unwrap();
        let cli = KeyStore::load(path).unwrap();
        let created = cli.create(new_key("Script")).unwrap();
        assert!(app.authenticate(&created.token).is_some());
        assert_eq!(app.list().len(), 1);
        assert!(cli.revoke(&created.key.id).unwrap());
        assert!(app.authenticate(&created.token).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn keys_survive_a_reload_without_storing_secrets() {
        let dir =
            std::env::temp_dir().join(format!("calcine-keys-{}", hex::encode(random_bytes::<4>())));
        let path = dir.join("api-keys.json");
        let token = {
            let store = KeyStore::load(path.clone()).unwrap();
            store.create(new_key("Open WebUI")).unwrap().token
        };
        let file = std::fs::read_to_string(&path).unwrap();
        assert!(
            !file.contains(&token),
            "the token must not be stored in clear"
        );

        let reloaded = KeyStore::load(path).unwrap();
        assert!(matches!(
            reloaded.authenticate(&token),
            Some(Caller::App(_))
        ));
        let _ = std::fs::remove_dir_all(dir);
    }
}
