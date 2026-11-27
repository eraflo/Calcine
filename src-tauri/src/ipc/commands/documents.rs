//! Files attached in the chat: documents, read into passages, and images,
//! handed to the webview to resize.

use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use calcine_core::Services;
use calcine_docs::{DocumentContext, SUPPORTED_EXTENSIONS};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::documents::{Documents, Imported};
use crate::ipc::error::{ApiError, ApiResult};

/// Images vision models take, with their media types.
const IMAGE_TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
    ("gif", "image/gif"),
    ("bmp", "image/bmp"),
];
/// Largest image read; it's resized before being sent anyway.
const MAX_IMAGE_BYTES: u64 = 30 * 1024 * 1024;

/// Choose files to attach: documents, and images when the model takes
/// them. Empty when the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn pick_attachments(app: AppHandle, images: bool) -> Vec<String> {
    let documents: Vec<&str> = SUPPORTED_EXTENSIONS
        .iter()
        .flat_map(|(_, extensions)| extensions.iter().copied())
        .collect();
    let pictures: Vec<&str> = IMAGE_TYPES
        .iter()
        .map(|(extension, _)| *extension)
        .collect();
    let mut dialog = app.dialog().file().set_title("Attach files");
    if images {
        let all: Vec<&str> = documents.iter().chain(&pictures).copied().collect();
        dialog = dialog
            .add_filter("Documents and images", &all)
            .add_filter("Images", &pictures);
    }
    dialog = dialog.add_filter("Documents", &documents);
    let (sender, receiver) = tokio::sync::oneshot::channel();
    dialog.pick_files(move |paths| {
        let _ = sender.send(paths);
    });
    receiver
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
        .into_iter()
        .map(|path| path.to_string())
        .collect()
}

/// Read a document's text and keep it for the chat. Reading failures come
/// back as [`Imported::Failed`], with a reason the chat can explain.
#[tauri::command]
#[specta::specta]
pub async fn import_document(documents: State<'_, Documents>, path: String) -> ApiResult<Imported> {
    Ok(documents.import(Path::new(&path)).await)
}

/// An image file as a data URL, for the webview to resize.
#[tauri::command]
#[specta::specta]
pub async fn read_image(path: String) -> ApiResult<String> {
    let path = Path::new(&path);
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let Some((_, media_type)) = IMAGE_TYPES.iter().find(|(known, _)| *known == extension) else {
        return Err(ApiError::invalid_input(format!(
            ".{extension} isn't an image"
        )));
    };
    let size = tokio::fs::metadata(path).await.map_err(ApiError::io)?.len();
    if size > MAX_IMAGE_BYTES {
        return Err(ApiError::invalid_input("this image is too large".into()));
    }
    let bytes = tokio::fs::read(path).await.map_err(ApiError::io)?;
    Ok(format!("data:{media_type};base64,{}", BASE64.encode(bytes)))
}

/// What `model` reads of these documents for the user's last messages
/// (newest first), within `budget` tokens.
#[tauri::command]
#[specta::specta]
pub async fn document_context(
    services: State<'_, Services>,
    documents: State<'_, Documents>,
    model: String,
    ids: Vec<String>,
    queries: Vec<String>,
    budget: u32,
) -> ApiResult<DocumentContext> {
    Ok(documents
        .context(&services, &model, &ids, &queries, budget)
        .await?)
}

/// Forget documents (their conversation was deleted, or they were removed
/// from it).
#[tauri::command]
#[specta::specta]
pub async fn remove_documents(documents: State<'_, Documents>, ids: Vec<String>) -> ApiResult<()> {
    documents.remove(ids).await;
    Ok(())
}

/// Delete the documents no conversation uses any more; returns how many.
#[tauri::command]
#[specta::specta]
pub async fn retain_documents(
    documents: State<'_, Documents>,
    keep: Vec<String>,
) -> ApiResult<u32> {
    Ok(documents.retain(keep).await)
}
