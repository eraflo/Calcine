//! Running GenieX commands.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use calcine_core::{Error, Result};
use tokio::process::Command;

/// Default limit for short, non-interactive commands (`list`, `version`, …).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Hide the console window when spawning from the GUI on Windows.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A resolved GenieX executable plus the global flags Calcine always passes.
#[derive(Debug, Clone)]
pub struct Cli {
    binary: PathBuf,
    data_dir: Option<PathBuf>,
    timeout: Duration,
}

impl Cli {
    pub fn new(binary: PathBuf, data_dir: Option<PathBuf>) -> Self {
        Self {
            binary,
            data_dir,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Build the command for `args`, with the flags every invocation needs:
    /// `--skip-update` (Calcine handles updates itself), `--data-dir` when
    /// configured, and `NO_COLOR` so output stays parsable.
    ///
    /// Arguments are passed as discrete values, never through a shell.
    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.binary);
        command.arg("--skip-update");
        if let Some(data_dir) = &self.data_dir {
            command.arg("--data-dir").arg(data_dir);
        }
        command
            .args(args)
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        command
    }

    /// Run a short command to completion and return its stdout.
    pub async fn run(&self, args: &[&str]) -> Result<String> {
        let label = args.join(" ");
        let output = tokio::time::timeout(self.timeout, self.command(args).output())
            .await
            .map_err(|_| Error::Timeout {
                command: label.clone(),
                seconds: self.timeout.as_secs(),
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let message = [stderr.trim(), stdout.trim()]
                .into_iter()
                .find(|text| !text.is_empty())
                .map_or_else(|| format!("exited with {}", output.status), str::to_owned);
            return Err(Error::Command {
                command: label,
                message,
            });
        }

        String::from_utf8(output.stdout)
            .map_err(|_| Error::Parse(format!("`geniex {label}` printed invalid UTF-8")))
    }
}
