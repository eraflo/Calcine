//! `calcine-cli`: the API without the window, for servers, scripts and
//! terminals. Same services, keys, settings and data folder as the app.
//!
//! ```text
//! calcine-cli serve [--port N] [--network] [--network-port N]
//! calcine-cli keys list
//! calcine-cli keys create <name> [--manage] [--network] [--local-files]
//! calcine-cli keys revoke <id>
//! calcine-cli path add|remove|status
//! ```

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::Write as _;

use calcine_gateway::{KeyScope, NewApiKey, RequestEntry};

use crate::setup::{self, DataDirs};

const USAGE: &str = "\
Calcine: your local models through an OpenAI-compatible API, without the window.

Usage:
  calcine-cli serve [options]      Serve the API until Ctrl+C
      --port <port>                Port on this PC (default: the app's, 18181)
      --network                    Also answer other devices over HTTPS
      --network-port <port>        Port for other devices (default 18443)
  calcine-cli keys list            List the API keys
  calcine-cli keys create <name> [--manage] [--network] [--local-files]
                                   Create a key and print it, once
  calcine-cli keys revoke <id>     Revoke a key
  calcine-cli path add|remove|status
                                   Put calcine-cli on your PATH (the installer does)
  calcine-cli --version

Settings, keys and models are the app's own. --port and --network only
apply to this run.";

/// Run a command; the process exit code.
pub fn main(args: &[String]) -> i32 {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match words.as_slice() {
        [] | ["help" | "-h" | "--help", ..] => {
            println!("{USAGE}");
            Ok(())
        }
        ["-V" | "--version" | "version"] => {
            println!("calcine-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        ["serve", options @ ..] => ServeOptions::parse(options).and_then(|options| serve(&options)),
        ["keys", rest @ ..] => keys(rest),
        ["path", action] => path(action),
        [other, ..] => Err(format!("unknown command {other}. Run calcine-cli --help.")),
    };
    match result {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("calcine-cli: {message}");
            1
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ServeOptions {
    port: Option<u16>,
    network: bool,
    network_port: Option<u16>,
}

impl ServeOptions {
    fn parse(options: &[&str]) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut words = options.iter();
        while let Some(word) = words.next() {
            let mut port = |name: &str| {
                words
                    .next()
                    .and_then(|value| value.parse::<u16>().ok())
                    .filter(|port| *port > 0)
                    .ok_or_else(|| format!("{name} needs a port between 1 and 65535"))
            };
            match *word {
                "--port" => parsed.port = Some(port("--port")?),
                "--network" => parsed.network = true,
                "--network-port" => parsed.network_port = Some(port("--network-port")?),
                other => return Err(format!("unknown option {other}. Run calcine-cli --help.")),
            }
        }
        Ok(parsed)
    }
}

fn data_dirs() -> Result<DataDirs, String> {
    DataDirs::from_env().ok_or_else(|| "can't find the user's data folder".to_owned())
}

fn serve(options: &ServeOptions) -> Result<(), String> {
    let dirs = data_dirs()?;
    let ServeOptions {
        port,
        network,
        network_port,
    } = *options;
    tauri::async_runtime::block_on(async move {
        let core = setup::build(&dirs, None).await;
        let gateway = core.gateway;
        gateway.override_settings(|settings| {
            if let Some(port) = port {
                settings.port = port;
            }
            if network {
                settings.network_enabled = true;
            }
            if let Some(port) = network_port {
                settings.network_port = port;
            }
        });
        if let Err(err) = core.services.runtime.info().await
            && core.services.backend == calcine_core::BackendKind::Geniex
        {
            return Err(format!(
                "GenieX isn't ready ({err}). Open Calcine once: it installs GenieX for you."
            ));
        }
        if let Err(message) = gateway.start().await {
            return Err(if message.contains("already used") {
                format!("{message}\nIf the Calcine window is open, it already serves the API.")
            } else {
                message
            });
        }
        announce(&gateway.status());
        let mut requests = gateway.subscribe_requests();
        let mut printed = HashSet::new();
        // Made once, so a signal between two requests isn't missed.
        let stop = stop_requested();
        tokio::pin!(stop);
        loop {
            tokio::select! {
                () = &mut stop => break,
                entry = requests.recv() => {
                    if let Ok(entry) = entry
                        && entry.duration_ms.is_some()
                        && printed.insert(entry.id)
                    {
                        // Only recent requests can still be updated.
                        printed.retain(|id: &u32| id + 1000 > entry.id);
                        println!("{}", request_line(&entry));
                    }
                }
            }
        }
        println!("Stopping…");
        gateway.stop().await;
        let _ = core.services.server.stop().await;
        Ok(())
    })
}

/// Ctrl+C, and on Windows also Ctrl+Break and closing the console window.
async fn stop_requested() {
    #[cfg(windows)]
    {
        use tokio::signal::windows;
        let (Ok(mut ctrl_break), Ok(mut close)) = (windows::ctrl_break(), windows::ctrl_close())
        else {
            let _ = tokio::signal::ctrl_c().await;
            return;
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = ctrl_break.recv() => {}
            _ = close.recv() => {}
        }
    }
    #[cfg(not(windows))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

fn announce(status: &calcine_gateway::GatewayStatus) {
    println!("Calcine API on {}", status.base_url);
    if let Some(network) = &status.network {
        println!("Other devices: {}", network.base_url);
        println!("  certificate SHA-256 {}", network.fingerprint);
        println!("  certificate file    {}", network.certificate_path);
    } else if let Some(error) = &status.network_error {
        println!("Other devices: not listening ({error})");
    }
    if status.require_api_key {
        println!("Apps need an API key: calcine-cli keys create <name>");
    }
    println!("Models load on the first request. Press Ctrl+C to stop.");
    let _ = std::io::stdout().flush();
}

/// `200 POST /v1/chat/completions qualcomm/Qwen3-4B 1.2 s 45 tok/s (App)`.
fn request_line(entry: &RequestEntry) -> String {
    let mut line = format!(
        "{} {} {}",
        entry
            .status
            .map_or_else(|| "---".to_owned(), |status| status.to_string()),
        entry.method,
        entry.path
    );
    if let Some(model) = &entry.model {
        line.push(' ');
        line.push_str(model);
    }
    if let Some(ms) = entry.duration_ms {
        #[allow(clippy::cast_precision_loss)]
        let seconds = ms as f64 / 1000.0;
        let _ = write!(line, " {seconds:.1} s");
    }
    if let Some(speed) = entry.tokens_per_second {
        let _ = write!(line, " {speed:.0} tok/s");
    }
    let _ = write!(line, " ({})", entry.client);
    if let Some(error) = &entry.error {
        let _ = write!(line, ": {error}");
    }
    line
}

/// Put the folder holding `calcine-cli` on the user's `PATH`, or take it off.
#[cfg(windows)]
fn path(action: &str) -> Result<(), String> {
    const NAME: &str = "Path";
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let dir = exe
        .parent()
        .ok_or("can't tell where calcine-cli is")?
        .display()
        .to_string();
    match action {
        "add" => {
            if crate::user_path::add(NAME, &dir)? {
                println!("Added {dir} to your PATH. New terminals will find calcine-cli.");
            } else {
                println!("{dir} is already on your PATH.");
            }
        }
        "remove" => {
            if crate::user_path::remove(NAME, &dir)? {
                println!("Removed {dir} from your PATH.");
            } else {
                println!("{dir} wasn't on your PATH.");
            }
        }
        "status" => {
            if crate::user_path::is_added(NAME, &dir)? {
                println!("{dir} is on your PATH.");
            } else {
                println!("{dir} isn't on your PATH. Add it with: calcine-cli path add");
            }
        }
        other => return Err(format!("unknown action {other}: add, remove or status")),
    }
    Ok(())
}

#[cfg(not(windows))]
fn path(_action: &str) -> Result<(), String> {
    Err(
        "on Linux, link calcine-cli into a folder on your PATH, e.g. \
         ln -s \"$(command -v calcine-cli)\" ~/.local/bin/"
            .into(),
    )
}

fn keys(words: &[&str]) -> Result<(), String> {
    let store = calcine_gateway::KeyStore::load(data_dirs()?.data.join("api-keys.json"))
        .map_err(|err| err.to_string())?;
    match words {
        ["list"] | [] => {
            let keys = store.list();
            if keys.is_empty() {
                println!("No API keys. Create one with: calcine-cli keys create <name>");
            }
            for key in keys {
                let mut rights: Vec<&str> = key
                    .scopes
                    .iter()
                    .map(|scope| match scope {
                        KeyScope::Inference => "run models",
                        KeyScope::Manage => "manage models",
                    })
                    .collect();
                if key.network {
                    rights.push("other devices");
                }
                if key.allow_local_files {
                    rights.push("local files");
                }
                println!(
                    "{}  {}  {}  {}",
                    key.id,
                    key.preview,
                    key.name,
                    rights.join(", ")
                );
            }
            Ok(())
        }
        ["create", name, flags @ ..] => {
            let mut scopes = vec![KeyScope::Inference];
            let mut network = false;
            let mut allow_local_files = false;
            for flag in flags {
                match *flag {
                    "--manage" => scopes.push(KeyScope::Manage),
                    "--network" => network = true,
                    "--local-files" => allow_local_files = true,
                    other => return Err(format!("unknown option {other}")),
                }
            }
            let created = store
                .create(NewApiKey {
                    name: (*name).to_owned(),
                    scopes,
                    allow_local_files,
                    network,
                })
                .map_err(|err| err.to_string())?;
            println!("{}", created.token);
            eprintln!(
                "Key \"{}\" created ({}). It's shown only now: keep it somewhere safe.",
                created.key.name, created.key.id
            );
            Ok(())
        }
        ["revoke", id] => {
            if store.revoke(id).map_err(|err| err.to_string())? {
                println!("Revoked {id}.");
                Ok(())
            } else {
                Err(format!(
                    "there's no key {id}. List them with: calcine-cli keys list"
                ))
            }
        }
        _ => Err("usage: calcine-cli keys list | create <name> [...] | revoke <id>".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_serve_options() {
        assert_eq!(ServeOptions::parse(&[]), Ok(ServeOptions::default()));
        assert_eq!(
            ServeOptions::parse(&["--port", "18200", "--network", "--network-port", "9443"]),
            Ok(ServeOptions {
                port: Some(18200),
                network: true,
                network_port: Some(9443),
            })
        );
        assert!(ServeOptions::parse(&["--port"]).is_err());
        assert!(ServeOptions::parse(&["--port", "0"]).is_err());
        assert!(ServeOptions::parse(&["--verbose"]).is_err());
    }

    #[test]
    fn prints_one_line_per_request() {
        let entry = RequestEntry {
            id: 1,
            started_at_ms: 0,
            client: "Phone".into(),
            method: "POST".into(),
            path: "/v1/chat/completions".into(),
            model: Some("qualcomm/Qwen3-4B".into()),
            stream: true,
            status: Some(200),
            duration_ms: Some(1300),
            first_token_ms: Some(150),
            prompt_tokens: Some(20),
            completion_tokens: Some(40),
            tokens_per_second: Some(21.4),
            error: None,
        };
        assert_eq!(
            request_line(&entry),
            "200 POST /v1/chat/completions qualcomm/Qwen3-4B 1.3 s 21 tok/s (Phone)"
        );
    }

    #[test]
    fn unknown_commands_fail() {
        assert_eq!(main(&["frobnicate".into()]), 1);
        assert_eq!(main(&["--version".into()]), 0);
    }
}
