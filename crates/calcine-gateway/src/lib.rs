//! Calcine's local gateway.
//!
//! Listens on `127.0.0.1:18181` (GenieX's default port, so existing clients
//! work), checks every request (see [`security`]), and forwards inference to
//! `geniex serve`, which runs on a private port and starts on demand.
//!
//! - `/v1/*`: OpenAI-compatible API (models, chat, completions, …)
//! - `/calcine/v1/*`: management (download, remove, jobs) for keys with `manage`
//! - `response_format` (JSON, JSON Schema) on every model: see [`structured`]
//! - `/api/*`: Ollama's API, translated (see [`ollama`]), also on port 11434
//!   when turned on

mod caller;
mod context;
mod error;
pub mod keys;
pub mod log;
pub mod network;
pub mod ollama;
mod proxy;
mod routes;
pub mod security;
pub mod settings;
mod state;
pub mod structured;

use std::path::PathBuf;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use calcine_core::Services;
use calcine_core::runtime::ServerState;
use tokio::sync::{broadcast, oneshot, watch};

pub use crate::keys::{ApiKeyInfo, CreatedApiKey, KeyError, KeyScope, KeyStore, NewApiKey};
pub use crate::log::RequestEntry;
use crate::log::RequestLog;
use crate::security::Listener;
pub use crate::settings::GatewaySettings;
use crate::state::AppState;
pub use crate::state::{GatewayStatus, NetworkStatus};

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
    /// Where the network port's certificate is kept (`None`: no network
    /// port).
    pub network_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Gateway {
    services: Services,
    keys: Arc<KeyStore>,
    log: Arc<RequestLog>,
    settings: Arc<RwLock<GatewaySettings>>,
    settings_path: Option<PathBuf>,
    builtin_origins: Vec<String>,
    network_dir: Option<PathBuf>,
    status: watch::Sender<GatewayStatus>,
    running: tokio::sync::Mutex<Option<Running>>,
}

#[derive(Debug)]
struct Running {
    app: Arc<AppState>,
    shutdown: oneshot::Sender<()>,
    /// Mirrors inference-server state into the status for this port.
    watcher: tokio::task::AbortHandle,
    /// Stops the Ollama port, when it's on.
    ollama: Option<oneshot::Sender<()>>,
    /// Stops the network port, when it's on.
    network: Option<network::Serving>,
}

impl Running {
    fn shut_down(self) {
        self.watcher.abort();
        let _ = self.shutdown.send(());
        if let Some(ollama) = self.ollama {
            let _ = ollama.send(());
        }
        if let Some(network) = self.network {
            network.stop();
        }
    }
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
            ollama_url: None,
            ollama_error: None,
            network: None,
            network_error: None,
        };
        Self {
            services,
            keys: options.keys,
            log: Arc::new(RequestLog::default()),
            settings: Arc::new(RwLock::new(settings)),
            settings_path: options.settings_path,
            builtin_origins: options.builtin_origins,
            network_dir: options.network_dir,
            status: watch::Sender::new(status),
            running: tokio::sync::Mutex::new(None),
        }
    }

    /// Change settings for this run only, without saving them. Call before
    /// [`Self::start`].
    pub fn override_settings(&self, change: impl FnOnce(&mut GatewaySettings)) {
        change(
            &mut self
                .settings
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }

    /// Start listening. Errors (port taken) are also reported in [`Self::status`].
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if running.is_some() {
            return Ok(());
        }
        let listener = match bind(self.settings().port).await {
            Ok(listener) => listener,
            Err(message) => {
                self.status.send_modify(|status| {
                    status.listening = false;
                    status.error = Some(message.clone());
                });
                return Err(message);
            }
        };
        *running = Some(self.serve(listener).await?);
        Ok(())
    }

    /// Serve the API on `listener`, and on the Ollama port when it's on.
    async fn serve(&self, listener: tokio::net::TcpListener) -> Result<Running, String> {
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
            status.base_url = format!("http://127.0.0.1:{port}/v1");
        });
        app.refresh_status();

        let shutdown = spawn_server(listener, routes::router(app.clone(), Listener::Main));
        let ollama = if self.settings().ollama_port_enabled {
            self.serve_ollama(&app).await
        } else {
            None
        };
        let network = if self.settings().network_enabled {
            self.serve_network(&app)
        } else {
            None
        };

        // Mirror inference-server transitions into the gateway status.
        let watcher = app.clone();
        let mut server_state = self.services.server.state();
        let watcher = tokio::spawn(async move {
            while server_state.changed().await.is_ok() {
                watcher.refresh_status();
            }
        })
        .abort_handle();

        tracing::info!(port, "gateway listening");
        Ok(Running {
            app,
            shutdown,
            watcher,
            ollama,
            network,
        })
    }

    /// Listen on Ollama's port with the same state (queue, log) as the main
    /// port. A taken port is reported in the status, not as an error.
    async fn serve_ollama(&self, app: &Arc<AppState>) -> Option<oneshot::Sender<()>> {
        let wanted = self.settings().ollama_port;
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", wanted)).await {
            Ok(listener) => listener,
            Err(err) => {
                let message = if err.kind() == std::io::ErrorKind::AddrInUse {
                    format!(
                        "Port {wanted} is already used, probably by Ollama. Quit Ollama to let Calcine answer Ollama apps there."
                    )
                } else {
                    format!("Couldn't listen on 127.0.0.1:{wanted}: {err}")
                };
                self.status.send_modify(|status| {
                    status.ollama_url = None;
                    status.ollama_error = Some(message);
                });
                return None;
            }
        };
        let port = listener.local_addr().map_or(wanted, |addr| addr.port());
        let stop = spawn_server(
            listener,
            routes::router(app.clone(), Listener::Ollama { port }),
        );
        self.status.send_modify(|status| {
            status.ollama_url = Some(format!("http://127.0.0.1:{port}"));
            status.ollama_error = None;
        });
        tracing::info!(port, "Ollama-compatible port listening");
        Some(stop)
    }

    /// Listen on the network port over HTTPS, with the same state as the
    /// main port. Problems (no certificate, port taken) go to the status.
    fn serve_network(&self, app: &Arc<AppState>) -> Option<network::Serving> {
        let fail = |message: String| {
            tracing::warn!(%message, "network port not listening");
            self.status.send_modify(|status| {
                status.network = None;
                status.network_error = Some(message);
            });
            None
        };
        let Some(dir) = &self.network_dir else {
            return fail("there's no folder to keep the certificate in".into());
        };
        let names = network::local_names();
        let identity = match network::tls::Identity::load_or_create(dir, &names) {
            Ok(identity) => identity,
            Err(message) => return fail(message),
        };
        let acceptor = match identity.acceptor() {
            Ok(acceptor) => acceptor,
            Err(message) => return fail(message),
        };
        let wanted = self.settings().network_port;
        // Bound synchronously so a taken port shows at once.
        let listener = match std::net::TcpListener::bind(("0.0.0.0", wanted)).and_then(|listener| {
            listener.set_nonblocking(true)?;
            tokio::net::TcpListener::from_std(listener)
        }) {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => {
                return fail(format!(
                    "Port {wanted} is already used by another program. Choose another one."
                ));
            }
            Err(err) => return fail(format!("Couldn't listen on port {wanted}: {err}")),
        };
        let port = listener.local_addr().map_or(wanted, |addr| addr.port());
        let serving = network::spawn(
            listener,
            acceptor,
            routes::router(app.clone(), Listener::Network { port }),
        );
        let host = network::local_address()
            .map(|address| address.to_string())
            .or_else(|| names.first().cloned())
            .unwrap_or_else(|| "localhost".into());
        self.status.send_modify(|status| {
            status.network = Some(NetworkStatus {
                base_url: format!("https://{host}:{port}/v1"),
                host_name: names.first().cloned(),
                fingerprint: identity.fingerprint(),
                certificate_path: network::tls::Identity::pem_path(dir).display().to_string(),
            });
            status.network_error = None;
        });
        tracing::info!(port, "network port listening");
        Some(serving)
    }

    /// Turn the network port on or off, on `port`, for the addresses in
    /// `allowed` (empty: private networks). Takes effect immediately.
    pub async fn set_network(
        &self,
        enabled: bool,
        port: u16,
        allowed: Vec<String>,
    ) -> Result<(), String> {
        let allowed: Vec<String> = allowed
            .iter()
            .map(|range| range.trim().to_owned())
            .filter(|range| !range.is_empty())
            .collect();
        for range in &allowed {
            network::allow::Range::parse(range)?;
        }
        if port == 0 {
            return Err("choose a port between 1 and 65535".into());
        }
        self.update_settings(|settings| {
            settings.network_enabled = enabled;
            settings.network_port = port;
            settings.network_allowed = allowed;
        })
        .map_err(|err| err.to_string())?;
        self.restart_network().await;
        Ok(())
    }

    /// Make a new certificate (other devices must trust it again), and use
    /// it right away.
    pub async fn renew_certificate(&self) -> Result<(), String> {
        let dir = self
            .network_dir
            .as_ref()
            .ok_or("there's no folder to keep the certificate in")?;
        network::tls::Identity::create(dir, &network::local_names())?;
        self.restart_network().await;
        Ok(())
    }

    async fn restart_network(&self) {
        let mut running = self.running.lock().await;
        let Some(current) = running.as_mut() else {
            return;
        };
        if let Some(serving) = current.network.take() {
            serving.stopped().await;
        }
        self.status.send_modify(|status| {
            status.network = None;
            status.network_error = None;
        });
        if self.settings().network_enabled {
            let app = current.app.clone();
            current.network = self.serve_network(&app);
        }
    }

    /// Answer Ollama apps on port 11434, or stop. Takes effect immediately.
    pub async fn set_ollama_port(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|settings| settings.ollama_port_enabled = enabled)
            .map_err(|err| err.to_string())?;
        let mut running = self.running.lock().await;
        let Some(current) = running.as_mut() else {
            return Ok(());
        };
        if let Some(stop) = current.ollama.take() {
            let _ = stop.send(());
        }
        self.status.send_modify(|status| {
            status.ollama_url = None;
            status.ollama_error = None;
        });
        if enabled {
            let app = current.app.clone();
            current.ollama = self.serve_ollama(&app).await;
        }
        Ok(())
    }

    pub async fn stop(&self) {
        if let Some(running) = self.running.lock().await.take() {
            running.shut_down();
        }
        self.status.send_modify(|status| {
            status.listening = false;
            status.ollama_url = None;
        });
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

    /// Listen on another loopback port. The gateway restarts; if the new
    /// port can't be used, the previous one is restored and the error is
    /// returned.
    pub async fn set_port(&self, port: u16) -> Result<(), String> {
        if port < 1024 {
            return Err("choose a port between 1024 and 65535".into());
        }
        let mut running = self.running.lock().await;
        if self.settings().port == port && running.is_some() {
            return Ok(());
        }
        // Take the new port before letting go of the old one, so a taken
        // port leaves the API where it was.
        let listener = bind(port).await?;
        if let Some(previous) = running.take() {
            previous.shut_down();
        }
        self.update_settings(|settings| settings.port = port)
            .map_err(|err| err.to_string())?;
        *running = Some(self.serve(listener).await?);
        Ok(())
    }

    /// Browser origins allowed to call the API besides Calcine's own UI,
    /// e.g. `http://localhost:3000` for a local web app. Takes effect
    /// immediately.
    pub fn set_allowed_origins(&self, origins: Vec<String>) -> Result<(), String> {
        let mut cleaned = Vec::with_capacity(origins.len());
        for origin in origins {
            let origin = origin.trim().trim_end_matches('/').to_ascii_lowercase();
            if origin.is_empty() {
                continue;
            }
            if !security::is_origin(&origin) {
                return Err(format!(
                    "{origin} isn't an origin: use the form http://host:port, without a path"
                ));
            }
            if !cleaned.contains(&origin) {
                cleaned.push(origin);
            }
        }
        self.update_settings(|settings| settings.allowed_origins = cleaned)
            .map_err(|err| err.to_string())
    }

    fn update_settings(&self, change: impl FnOnce(&mut GatewaySettings)) -> std::io::Result<()> {
        let settings = {
            let mut settings = self
                .settings
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            change(&mut settings);
            settings.clone()
        };
        if let Some(path) = &self.settings_path {
            settings.save(path)?;
        }
        Ok(())
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

/// Serve `router` on `listener` until the returned sender fires.
fn spawn_server(listener: tokio::net::TcpListener, router: axum::Router) -> oneshot::Sender<()> {
    let (shutdown, stop) = oneshot::channel::<()>();
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
    shutdown
}

async fn bind(port: u16) -> Result<tokio::net::TcpListener, String> {
    tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|err| {
            if err.kind() == std::io::ErrorKind::AddrInUse {
                format!(
                    "Port {port} is already used by another program, possibly `geniex serve` \
                     started from a terminal. Stop it, or change the port in Settings."
                )
            } else {
                format!("Couldn't listen on 127.0.0.1:{port}: {err}")
            }
        })
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
