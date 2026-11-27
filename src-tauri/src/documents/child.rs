//! Reading a file in a child process: this same executable, started with
//! [`CHILD_ARG`], writes the text as JSON and exits.

use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use calcine_docs::{DocumentProblem, Extracted, PdfReader};
use serde::{Deserialize, Serialize};

/// First argument of the child process that reads a file:
/// `calcine --extract-document <detailed|simple> <file> <output.json>`.
const CHILD_ARG: &str = "--extract-document";
/// A file that takes longer than this to read is given up on.
const READ_TIMEOUT: Duration = Duration::from_secs(120);

/// What the child process writes.
#[derive(Debug, Serialize, Deserialize)]
pub(super) enum ChildOutput {
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

/// Run the reader in a child process; `Err` when it crashed or hung.
pub(super) async fn read_in_child(path: &Path, reader: PdfReader) -> Result<ChildOutput, String> {
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
