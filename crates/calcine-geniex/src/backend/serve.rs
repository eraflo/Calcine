//! `geniex serve`, supervised: started on demand on a free loopback port,
//! health-checked, logged, and restarted after a crash on the next request.

use std::collections::VecDeque;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use calcine_core::runtime::{InferenceServer, ServerOptions, ServerState};
use calcine_core::{Error, Result};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::{oneshot, watch};

use crate::Geniex;

/// Output lines kept for the Server page.
const LOG_LINES: usize = 500;
/// Loading the runtime plugins takes a few seconds on first start.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
const HEALTH_INTERVAL: Duration = Duration::from_millis(250);

/// The supervised `geniex serve` process.
#[derive(Debug)]
pub struct GeniexServer {
    geniex: Geniex,
    options: Mutex<ServerOptions>,
    running: tokio::sync::Mutex<Option<Running>>,
    state: watch::Sender<ServerState>,
    logs: Arc<Mutex<VecDeque<String>>>,
    http: reqwest::Client,
}

#[derive(Debug)]
struct Running {
    url: String,
    exited: Arc<AtomicBool>,
    stop: Option<oneshot::Sender<()>>,
}

impl GeniexServer {
    pub fn new(geniex: Geniex, options: ServerOptions) -> Self {
        Self {
            geniex,
            options: Mutex::new(options),
            running: tokio::sync::Mutex::new(None),
            state: watch::Sender::new(ServerState::Stopped),
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(LOG_LINES))),
            http: reqwest::Client::new(),
        }
    }

    async fn start(&self) -> Result<Running> {
        let port = free_port()?;
        let host = format!("127.0.0.1:{port}");
        let options = self.options();
        let keepalive = options.keepalive_secs.to_string();
        let context_size = options.context_size.to_string();
        // The server is internal: only the gateway talks to it, never a browser.
        let args = [
            "serve",
            "--host",
            &host,
            "--origins",
            "http://127.0.0.1",
            "--keepalive",
            &keepalive,
            "--nctx",
            &context_size,
        ];
        let mut child = self.geniex.cli()?.command(&args).spawn()?;
        tracing::info!(%host, "starting geniex serve");

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(collect_lines(stdout, self.logs.clone()));
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(collect_lines(stderr, self.logs.clone()));
        }

        let exited = Arc::new(AtomicBool::new(false));
        let (stop_tx, stop_rx) = oneshot::channel();
        tokio::spawn(watch_process(
            child,
            stop_rx,
            exited.clone(),
            self.state.clone(),
            self.logs.clone(),
        ));

        let url = format!("http://{host}");
        let deadline = Instant::now() + STARTUP_TIMEOUT;
        loop {
            if exited.load(Ordering::SeqCst) {
                return Err(Error::Command {
                    command: "serve".into(),
                    message: last_lines(&self.logs, 6),
                });
            }
            let healthy = self
                .http
                .get(format!("{url}/v1/"))
                .timeout(Duration::from_secs(2))
                .send()
                .await
                .is_ok_and(|response| response.status().is_success());
            if healthy {
                break;
            }
            if Instant::now() >= deadline {
                let _ = stop_tx.send(());
                return Err(Error::Timeout {
                    command: "serve".into(),
                    seconds: STARTUP_TIMEOUT.as_secs(),
                });
            }
            tokio::time::sleep(HEALTH_INTERVAL).await;
        }

        self.state.send_replace(ServerState::Ready {
            url: url.clone(),
            started_at_ms: now_ms(),
        });
        tracing::info!(%url, "geniex serve is ready");
        Ok(Running {
            url,
            exited,
            stop: Some(stop_tx),
        })
    }
}

#[async_trait]
impl InferenceServer for GeniexServer {
    async fn ensure_running(&self) -> Result<String> {
        // Holding the lock across start-up makes concurrent callers share it.
        let mut running = self.running.lock().await;
        if let Some(current) = running
            .as_ref()
            .filter(|r| !r.exited.load(Ordering::SeqCst))
        {
            return Ok(current.url.clone());
        }
        self.state.send_replace(ServerState::Starting);
        match self.start().await {
            Ok(started) => {
                let url = started.url.clone();
                *running = Some(started);
                Ok(url)
            }
            Err(err) => {
                self.state.send_replace(ServerState::Failed {
                    message: err.to_string(),
                });
                *running = None;
                Err(err)
            }
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(mut current) = self.running.lock().await.take()
            && let Some(stop) = current.stop.take()
        {
            let _ = stop.send(());
        }
        self.state.send_replace(ServerState::Stopped);
        Ok(())
    }

    fn state(&self) -> watch::Receiver<ServerState> {
        self.state.subscribe()
    }

    fn logs(&self) -> Vec<String> {
        self.logs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }

    fn options(&self) -> ServerOptions {
        *self.options.lock().unwrap_or_else(PoisonError::into_inner)
    }

    async fn set_options(&self, options: ServerOptions) -> Result<()> {
        options.validate().map_err(Error::InvalidInput)?;
        *self.options.lock().unwrap_or_else(PoisonError::into_inner) = options;
        let was_running = self
            .running
            .lock()
            .await
            .as_ref()
            .is_some_and(|r| !r.exited.load(Ordering::SeqCst));
        if was_running {
            self.stop().await?;
            self.ensure_running().await?;
        }
        Ok(())
    }
}

/// Wait for the process to exit or for a stop request, and report a crash.
async fn watch_process(
    mut child: tokio::process::Child,
    stop: oneshot::Receiver<()>,
    exited: Arc<AtomicBool>,
    state: watch::Sender<ServerState>,
    logs: Arc<Mutex<VecDeque<String>>>,
) {
    tokio::select! {
        status = child.wait() => {
            exited.store(true, Ordering::SeqCst);
            let detail = status.map_or_else(|err| err.to_string(), |status| status.to_string());
            tracing::warn!(%detail, "geniex serve exited");
            // Only a crash while ready or starting is a failure; a stop already set Stopped.
            state.send_if_modified(|current| {
                if matches!(current, ServerState::Stopped) {
                    return false;
                }
                *current = ServerState::Failed {
                    message: format!("geniex serve exited ({detail}). {}", last_lines(&logs, 4)),
                };
                true
            });
        }
        _ = stop => {
            let _ = child.kill().await;
            exited.store(true, Ordering::SeqCst);
            tracing::info!("geniex serve stopped");
        }
    }
}

async fn collect_lines(pipe: impl AsyncRead + Unpin, logs: Arc<Mutex<VecDeque<String>>>) {
    let mut lines = BufReader::new(pipe).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let mut logs = logs.lock().unwrap_or_else(PoisonError::into_inner);
        if logs.len() == LOG_LINES {
            logs.pop_front();
        }
        logs.push_back(line);
    }
}

fn last_lines(logs: &Mutex<VecDeque<String>>, count: usize) -> String {
    let logs = logs.lock().unwrap_or_else(PoisonError::into_inner);
    let skip = logs.len().saturating_sub(count);
    logs.iter()
        .skip(skip)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ask the OS for a free loopback port.
fn free_port() -> Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
