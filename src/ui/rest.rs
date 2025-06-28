use std::sync::Arc;

use axum::{ extract::{Path, State}, http::StatusCode, response::{Html, IntoResponse}, Json };
use serde::Deserialize;
use serde_json::json;
use similar::{ ChangeTag, TextDiff };
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{model::CaptureV2_4, sinks::UiSource};

use super::ProdlogUiState;

#[derive(Deserialize)]
pub struct EntryPostData {
    pub uuid: String,
    pub message: String,
    pub is_noop: bool,
}

#[derive(Deserialize)]
pub struct EntryRedactData {
    pub uuid: String,
    pub password: String,
}

pub async fn get_entry(
    sink: Arc<RwLock<Box<dyn UiSource>>>,
    uuid: &str,
) -> Result<CaptureV2_4, (StatusCode, String)> {
    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((StatusCode::BAD_REQUEST, "Invalid UUID format".to_string()));
        }
    };

    let entry = match sink.read().await.get_entry_by_id(uuid) {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "Entry not found".to_string()));
        }
        Err(err) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error loading entry: {}", err)));
        }
    };

    Ok(entry)
}

pub async fn handle_entry_get(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    match get_entry(sink.clone(), &uuid).await {
        Ok(entry) => (StatusCode::OK, Json(entry)).into_response(),
        Err((status, message)) => (status, Json(json!({ "error": message }))).into_response(),
    }
}

pub async fn handle_entry_post(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<EntryPostData>
) -> impl IntoResponse {
    let entry = match get_entry(sink.clone(), &data.uuid).await {
        Ok(mut entry) => {
                entry.message = data.message;
                entry.is_noop = data.is_noop;
                entry
        },
        Err((status, message)) => return (status, Json(json!({ "error": message }))).into_response(),
    };

    match sink.write().await.add_entry(&entry) {
        Ok(_) => (StatusCode::OK, Html(String::from("Success"))).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error saving entry: {}", err))).into_response(),
    }
}

fn simple_diff(orig: &str, edited: &str) -> String {
    let diff = TextDiff::from_lines(orig, edited);
    let mut html = String::new();
    for change in diff.iter_all_changes() {
        let (class, sign) = match change.tag() {
            ChangeTag::Delete => ("diff-del", "-"),
            ChangeTag::Insert => ("diff-ins", "+"),
            ChangeTag::Equal => ("", " "),
        };
        html.push_str(
            &format!(
                r#"<div class="{}"><span>{}</span>{}</div>"#,
                class,
                sign,
                html_escape::encode_text(change.value())
            )
        );
    }
    html
}

pub async fn handle_entry_redact(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<EntryRedactData>
) -> impl IntoResponse {
    if data.password.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Password cannot be empty" }))).into_response();
    }

    let mut entry = match get_entry(sink.clone(), &data.uuid).await {
        Ok(entry) => entry,
        Err((status, message)) => return (status, Json(json!({ "error": message }))).into_response(),
    };

    let password = data.password.trim();
    let mut redacted = false;

    // Redact password in command
    if entry.cmd.contains(password) {
        entry.cmd = entry.cmd.replace(password, "[REDACTED]");
        redacted = true;
    }

    // Redact password in captured output
    let output_str = String::from_utf8_lossy(&entry.captured_output);
    if output_str.contains(password) {
        let new_output = output_str.replace(password, "[REDACTED]");
        entry.captured_output = new_output.into_bytes();
        redacted = true;
    }

    // Redact password in original content (for edit entries)
    if !entry.original_content.is_empty() {
        let original_str = String::from_utf8_lossy(&entry.original_content);
        if original_str.contains(password) {
            let new_original = original_str.replace(password, "[REDACTED]");
            entry.original_content = new_original.into_bytes();
            redacted = true;
        }
    }

    // Redact password in edited content (for edit entries)
    if !entry.edited_content.is_empty() {
        let edited_str = String::from_utf8_lossy(&entry.edited_content);
        if edited_str.contains(password) {
            let new_edited = edited_str.replace(password, "[REDACTED]");
            entry.edited_content = new_edited.into_bytes();
            redacted = true;
        }
    }

    if !redacted {
        return (StatusCode::OK, Json(json!({ "message": "Password not found in this entry" }))).into_response();
    }

    // Save the redacted entry
    match sink.write().await.add_entry(&entry) {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Password redacted successfully" }))).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Error saving entry: {}", err) }))).into_response(),
    }
}

pub async fn handle_diffcontent(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    let entry = match get_entry(sink.clone(), &uuid).await {
        Ok(entry) => entry,
        Err((status, message)) => return (status, Json(json!({ "error": message }))).into_response(),
    };

    let orig = String::from_utf8_lossy(&entry.original_content);
    let edited = String::from_utf8_lossy(&entry.edited_content);
    (StatusCode::OK, Json(json!({ "diff": simple_diff(&orig, &edited) }))).into_response()
}

