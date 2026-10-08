//! Calcine's local gateway.
//!
//! Listens on `127.0.0.1:18181` (GenieX's default port, so existing clients
//! work), checks every request (see [`security`]), and forwards inference to
//! `geniex serve`, which runs on a private port and starts on demand.
//!
//! - `/v1/*`: OpenAI-compatible API (models, chat, completions, …)
//! - `/calcine/v1/*`: management (download, remove, jobs) for keys with `manage`

mod caller;
mod error;
pub mod keys;
pub mod log;
mod proxy;
mod routes;
pub mod security;
pub mod settings;
mod state;

use std::path::PathBuf;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use calcine_core::Services;
use calcine_core::runtime::ServerState;
use tokio::sync::{broadcast, oneshot, watch};

pub use crate::keys::{ApiKeyInfo, CreatedApiKey, KeyError, KeyScope, KeyStore, NewApiKey};
pub use crate::log::RequestEntry;
use crate::log::RequestLog;
pub use crate::settings::GatewaySettings;
use crate::state::AppState;
pub use crate::state::GatewayStatus;

/// Browser origins of Calcine's own webview (production and `tauri dev`).
pub const CALCINE_ORIGINS: &[&str] = &[
    "http://tauri.localhost",
    "https://tauri.localhost",
    "tauri://localhost",
];

#[derive(Debug)]
pub struct GatewayOptions {
    pub keys: Arc<KeyStore>,
    /// Where settings are persisted (`None`: defaults, in memory).
    pub settings_path: Option<PathBuf>,
    /// Origins always allowed (Calcine's own UI).
    pub builtin_origins: Vec<String>,
}

#[derive(Debug)]
pub struct Gateway {
    services: Services,
    keys: Arc<KeyStore>,
    log: Arc<RequestLog>,
    settings: Arc<RwLock<GatewaySettings>>,
    settings_path: Option<PathBuf>,
    builtin_origins: Vec<String>,
    status: watch::Sender<GatewayStatus>,
    running: tokio::sync::Mutex<Option<Running>>,
}

#[derive(Debug)]
struct Running {
    app: Arc<AppState>,
    shutdown: oneshot::Sender<()>,
}

impl Gateway {
    pub fn new(services: Services, options: GatewayOptions) -> Self {
        let settings = options
            .settings_path
            .as_deref()
            .map(GatewaySettings::load)
            .unwrap_or_default();
        let status = GatewayStatus {
            listening: false,
            base_url: format!("http://127.0.0.1:{}/v1", settings.port),
            error: None,
            server: ServerState::Stopped,
            active_requests: 0,
            queued_requests: 0,
            require_api_key: settings.require_api_key,
        };
        Self {
            services,
            keys: options.keys,
            log: Arc::new(RequestLog::default()),
            settings: Arc::new(RwLock::new(settings)),
            settings_path: options.settings_path,
            builtin_origins: options.builtin_origins,
            status: watch::Sender::new(status),
            running: tokio::sync::Mutex::new(None),
        }
    }

    /// Start listening. Errors (port taken) are also reported in [`Self::status`].
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if running.is_some() {
            return Ok(());
        }
        let port = self.settings().port;
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => listener,
            Err(err) => {
                let message = if err.kind() == std::io::ErrorKind::AddrInUse {
                    format!(
                        "Port {port} is already used by another program, possibly `geniex serve` \
                         started from a terminal. Stop it, or change the port in Settings."
                    )
                } else {
                    format!("Couldn't listen on 127.0.0.1:{port}: {err}")
                };
                self.status.send_modify(|status| {
                    status.listening = false;
                    status.error = Some(message.clone());
                });
                return Err(message);
            }
        };
        let port = listener.local_addr().map_err(|err| err.to_string())?.port();

        let app = Arc::new(AppState::new(
            self.services.clone(),
            self.keys.clone(),
            self.log.clone(),
            self.settings.clone(),
            self.builtin_origins.clone(),
            self.status.clone(),
            port,
        ));
        self.status.send_modify(|status| {
            status.listening = true;
            status.error = None;
        });
        app.refresh_status();

        let (shutdown, stop) = oneshot::channel::<()>();
        let router = routes::router(app.clone());
        tokio::spawn(async move {
            let served = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = stop.await;
                })
                .await;
            if let Err(err) = served {
                tracing::error!(%err, "gateway stopped unexpectedly");
            }
        });

        // Mirror inference-server transitions into the gateway status.
        let watcher = app.clone();
        let mut server_state = self.services.server.state();
        tokio::spawn(async move {
            while server_state.changed().await.is_ok() {
                watcher.refresh_status();
            }
        });

        tracing::info!(port, "gateway listening");
        *running = Some(Running { app, shutdown });
        Ok(())
    }

    pub async fn stop(&self) {
        if let Some(running) = self.running.lock().await.take() {
            let _ = running.shutdown.send(());
        }
        self.status.send_modify(|status| status.listening = false);
    }

    pub fn status(&self) -> GatewayStatus {
        self.status.borrow().clone()
    }

    pub fn subscribe_status(&self) -> watch::Receiver<GatewayStatus> {
        self.status.subscribe()
    }

    pub fn requests(&self) -> Vec<RequestEntry> {
        self.log.list()
    }

    pub fn subscribe_requests(&self) -> broadcast::Receiver<RequestEntry> {
        self.log.subscribe()
    }

    pub fn keys(&self) -> &Arc<KeyStore> {
        &self.keys
    }

    pub fn settings(&self) -> GatewaySettings {
        self.settings
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Require (or stop requiring) API keys. Takes effect immediately.
    pub async fn set_require_api_key(&self, require: bool) -> std::io::Result<()> {
        let settings = {
            let mut settings = self
                .settings
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            settings.require_api_key = require;
            settings.clone()
        };
        if let Some(path) = &self.settings_path {
            settings.save(path)?;
        }
        match self.running.lock().await.as_ref() {
            Some(running) => running.app.refresh_status(),
            None => self
                .status
                .send_modify(|status| status.require_api_key = require),
        }
        Ok(())
    }
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
