//! State shared by every request.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use calcine_core::Services;
use calcine_core::runtime::ServerState;
use serde::Serialize;
use specta::Type;
use tokio::sync::{Semaphore, watch};

use crate::keys::KeyStore;
use crate::log::RequestLog;
use crate::network::limiter::Limiter;
use crate::settings::GatewaySettings;

/// What the Server page shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GatewayStatus {
    pub listening: bool,
    /// Base URL apps use, e.g. `http://127.0.0.1:18181/v1`.
    pub base_url: String,
    /// Why the gateway isn't listening (port taken, …).
    pub error: Option<String>,
    pub server: ServerState,
    pub active_requests: u32,
    pub queued_requests: u32,
    pub require_api_key: bool,
    /// Where Ollama apps connect, when the Ollama port is on and listening.
    pub ollama_url: Option<String>,
    /// Why the Ollama port isn't listening (taken by Ollama itself, ...).
    pub ollama_error: Option<String>,
    /// Where other devices connect, when the network port is listening.
    pub network: Option<NetworkStatus>,
    /// Why the network port isn't listening.
    pub network_error: Option<String>,
}

/// The network port, as other devices see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    /// Base URL for other devices, e.g. `https://192.168.1.20:18443/v1`.
    pub base_url: String,
    /// This PC's name on the network, for clients that prefer it.
    pub host_name: Option<String>,
    /// SHA-256 of the certificate, for clients to pin.
    pub fingerprint: String,
    /// The certificate as a `.pem` file, for clients to import.
    pub certificate_path: String,
}

#[derive(Debug)]
pub struct AppState {
    pub services: Services,
    pub keys: Arc<KeyStore>,
    pub log: Arc<RequestLog>,
    pub http: reqwest::Client,
    /// GenieX runs one inference at a time; waiting here makes the queue visible.
    pub queue: Arc<Semaphore>,
    pub port: u16,
    /// Turns away network addresses that send too many invalid keys.
    pub limiter: Limiter,
    settings: Arc<RwLock<GatewaySettings>>,
    builtin_origins: Vec<String>,
    active: AtomicU32,
    queued: AtomicU32,
    status: watch::Sender<GatewayStatus>,
}

impl AppState {
    pub fn new(
        services: Services,
        keys: Arc<KeyStore>,
        log: Arc<RequestLog>,
        settings: Arc<RwLock<GatewaySettings>>,
        builtin_origins: Vec<String>,
        status: watch::Sender<GatewayStatus>,
        port: u16,
    ) -> Self {
        Self {
            services,
            keys,
            log,
            http: reqwest::Client::new(),
            queue: Arc::new(Semaphore::new(1)),
            port,
            limiter: Limiter::default(),
            settings,
            builtin_origins,
            active: AtomicU32::new(0),
            queued: AtomicU32::new(0),
            status,
        }
    }

    pub fn settings(&self) -> GatewaySettings {
        self.settings
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn origin_allowed(&self, origin: &str) -> bool {
        let origin = normalize_origin(origin);
        self.builtin_origins
            .iter()
            .any(|allowed| normalize_origin(allowed) == origin)
            || self
                .settings()
                .allowed_origins
                .iter()
                .any(|allowed| normalize_origin(allowed) == origin)
    }

    pub fn enter_queue(&self) {
        self.queued.fetch_add(1, Ordering::SeqCst);
        self.refresh_status();
    }

    pub fn leave_queue_and_start(&self) {
        self.queued.fetch_sub(1, Ordering::SeqCst);
        self.active.fetch_add(1, Ordering::SeqCst);
        self.refresh_status();
    }

    pub fn finish(&self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
        self.refresh_status();
    }

    /// Recompute the published status from the live counters and settings.
    pub fn refresh_status(&self) {
        let settings = self.settings();
        let server = self.services.server.state().borrow().clone();
        let active = self.active.load(Ordering::SeqCst);
        let queued = self.queued.load(Ordering::SeqCst);
        let port = self.port;
        self.status.send_modify(|status| {
            status.server = server;
            status.active_requests = active;
            status.queued_requests = queued;
            status.require_api_key = settings.require_api_key;
            status.base_url = format!("http://127.0.0.1:{port}/v1");
        });
    }
}

/// Decrements the active-request counter when a request ends, however it ends.
#[derive(Debug)]
pub struct ActiveRequest(pub Arc<AppState>);

impl Drop for ActiveRequest {
    fn drop(&mut self) {
        self.0.finish();
    }
}

fn normalize_origin(origin: &str) -> String {
    origin.trim().trim_end_matches('/').to_ascii_lowercase()
}
