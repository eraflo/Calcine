//! Downloading an installer with progress, verified by SHA-256.

use std::path::Path;
use std::time::Instant;

use calcine_core::jobs::{JobCtx, JobPhase, JobProgress};
use calcine_core::runtime::InstallerAsset;
use calcine_core::{Error, Result};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

/// Put `asset` at `destination`, reusing a previous download when its
/// checksum still matches. Cancellable while downloading.
pub async fn fetch(
    http: &reqwest::Client,
    asset: &InstallerAsset,
    destination: &Path,
    ctx: &JobCtx,
) -> Result<()> {
    if destination.is_file() && sha256_file(destination).await? == asset.sha256 {
        return Ok(());
    }
    if let Some(dir) = destination.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }

    let partial = destination.with_extension("partial");
    let result = download(http, asset, &partial, ctx).await;
    match result {
        Ok(digest) if digest == asset.sha256 => {
            tokio::fs::rename(&partial, destination).await?;
            Ok(())
        }
        Ok(digest) => {
            let _ = tokio::fs::remove_file(&partial).await;
            Err(Error::InvalidInput(format!(
                "the downloaded {} doesn't match the official checksum \
                 (expected {}, got {digest}); it was deleted",
                asset.name, asset.sha256
            )))
        }
        Err(err) => {
            let _ = tokio::fs::remove_file(&partial).await;
            Err(err)
        }
    }
}

/// Stream to `path`, hashing as we go. Returns the hex SHA-256.
async fn download(
    http: &reqwest::Client,
    asset: &InstallerAsset,
    path: &Path,
    ctx: &JobCtx,
) -> Result<String> {
    let response = http
        .get(&asset.url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|err| Error::Network(format!("couldn't download {}: {err}", asset.name)))?;
    let total = response.content_length().unwrap_or(asset.size);
    let mut file = tokio::fs::File::create(path).await?;
    let mut hasher = Sha256::new();
    let mut done = 0_u64;
    let started = Instant::now();
    let mut stream = response.bytes_stream();
    loop {
        let chunk = tokio::select! {
            chunk = stream.next() => chunk,
            () = ctx.cancelled() => return Err(Error::Cancelled),
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk.map_err(|err| {
            Error::Network(format!("the download of {} broke: {err}", asset.name))
        })?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        done += chunk.len() as u64;
        let elapsed = started.elapsed().as_secs_f64();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let speed = (elapsed > 0.5).then(|| (done as f64 / elapsed) as u64);
        ctx.report(JobProgress {
            done_bytes: done,
            total_bytes: Some(total),
            bytes_per_second: speed,
            phase: Some(JobPhase::Downloading),
            step: None,
        });
    }
    file.flush().await?;
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha256_file(path: &Path) -> Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 1 << 20];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(hex::encode(hasher.finalize()))
    })
    .await
    .map_err(|err| Error::Io(std::io::Error::other(err)))?
}
