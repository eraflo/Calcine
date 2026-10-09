//! `geniex pull` with live progress and cancellation.

use std::collections::VecDeque;

use calcine_core::jobs::JobCtx;
use calcine_core::models::{ModelType, PullRequest};
use calcine_core::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::cli::Cli;
use crate::parse::progress::{FrameSplitter, parse_frame};

/// Non-progress lines kept to explain a failure.
const KEPT_LINES: usize = 12;

/// What GenieX says before asking for the chipset.
const NO_CHIPSET: &str = "No chipset configured";

pub(crate) async fn run(cli: &Cli, request: &PullRequest, ctx: &JobCtx) -> Result<()> {
    let reference = request.reference.cli_arg();
    let mut args = vec!["pull", reference.as_str()];
    if let Some(hub) = request.reference.hub.cli_value() {
        args.extend(["--model-hub", hub]);
    }
    if request.is_import() {
        let path = request.local_path.as_deref().ok_or_else(|| {
            Error::InvalidInput("choose a folder or a .zip file to import".into())
        })?;
        args.extend(["--local-path", path]);
    }
    match request.model_type {
        Some(ModelType::Llm) => args.extend(["--model-type", "llm"]),
        Some(ModelType::Vlm) => args.extend(["--model-type", "vlm"]),
        Some(ModelType::Unknown) | None => {}
    }
    let label = args.join(" ");
    tracing::info!(command = %label, job = ctx.id(), "starting download");

    // Without a terminal GenieX skips its pickers and takes the recommended
    // precision, which is what an unattended download wants.
    let mut child = cli.command(&args).spawn()?;
    let stderr = child.stderr.take().ok_or_else(|| missing_pipe("stderr"))?;
    let stdout = child.stdout.take().ok_or_else(|| missing_pipe("stdout"))?;
    let stderr_task = tokio::spawn(follow_output(stderr, Some(ctx.clone())));
    let stdout_task = tokio::spawn(follow_output(stdout, None));

    let status = tokio::select! {
        status = child.wait() => status?,
        () = ctx.cancelled() => {
            // GenieX keeps partial files, so pulling again resumes.
            let _ = child.kill().await;
            return Err(Error::Cancelled);
        }
    };

    let stderr_lines = stderr_task.await.unwrap_or_default();
    let stdout_lines = stdout_task.await.unwrap_or_default();
    if status.success() {
        return Ok(());
    }
    // Without a terminal GenieX can't ask which chipset this is (on Linux
    // it often can't tell). It says so on stdout, then fails on stderr.
    if stdout_lines
        .iter()
        .chain(&stderr_lines)
        .any(|line| line.contains(NO_CHIPSET))
    {
        return Err(Error::InvalidInput(
            "GenieX doesn't know which Snapdragon this is. Choose your chipset in \
             Hardware › Chipset (or run calcine-cli geniex chipset), then download again."
                .into(),
        ));
    }
    let message = [stderr_lines, stdout_lines]
        .into_iter()
        .find(|lines| !lines.is_empty())
        .map_or_else(
            || format!("exited with {status}"),
            |lines| lines.into_iter().collect::<Vec<_>>().join("\n"),
        );
    Err(Error::Command {
        command: label,
        message,
    })
}

/// Read a pipe to the end, reporting progress frames to `ctx` and returning
/// the last few other lines.
async fn follow_output(mut pipe: impl AsyncRead + Unpin, ctx: Option<JobCtx>) -> VecDeque<String> {
    let mut splitter = FrameSplitter::default();
    let mut kept = VecDeque::with_capacity(KEPT_LINES);
    let mut buffer = [0_u8; 4096];

    let mut handle = |frame: String| match (parse_frame(&frame), &ctx) {
        (Some(progress), Some(ctx)) => ctx.report(progress),
        (Some(_), None) => {}
        (None, _) => {
            if kept.len() == KEPT_LINES {
                kept.pop_front();
            }
            kept.push_back(frame.trim().to_owned());
        }
    };

    loop {
        match pipe.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(read) => splitter
                .push(&buffer[..read])
                .into_iter()
                .for_each(&mut handle),
        }
    }
    if let Some(rest) = splitter.finish() {
        handle(rest);
    }
    kept
}

fn missing_pipe(name: &str) -> Error {
    Error::Io(std::io::Error::other(format!(
        "couldn't read GenieX {name}"
    )))
}
