use std::sync::Arc;

use axum::{ extract::{Path, State}, http::StatusCode, response::{Html, IntoResponse}, Json };
use serde::Deserialize;
use serde_json::json;
use similar::{ ChangeTag, TextDiff };
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{model::CaptureV2_4, sinks::{UiSource, Filters}, helpers::redact_passwords_from_entry};

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

#[derive(Deserialize)]
pub struct BulkRedactData {
    pub passwords: Vec<String>,
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

pub async fn handle_entry_redact_post(
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
    
    // Use the helper function to redact the password
    let redacted = redact_passwords_from_entry(&mut entry, &[password.to_string()]);

    if !redacted {
        return (StatusCode::OK, Json(json!({ "message": "Password not found in this entry" }))).into_response();
    }

    // Save the redacted entry
    match sink.write().await.add_entry(&entry) {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Password redacted successfully" }))).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Error saving entry: {}", err) }))).into_response(),
    }
}

pub async fn handle_bulk_redact_post(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<BulkRedactData>
) -> impl IntoResponse {
    if data.passwords.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No passwords provided" }))).into_response();
    }

    // Filter out empty passwords
    let passwords: Vec<String> = data.passwords
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();

    if passwords.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No valid passwords provided" }))).into_response();
    }

    // Get all entries
    let entries = match sink.read().await.get_entries(&Filters::default()) {
        Ok(entries) => entries,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Error loading entries: {}", e) }))).into_response(),
    };

    let mut redacted_count = 0;
    let total_entries = entries.len();

    // Process each entry
    for entry in entries {
        let mut modified_entry = entry.clone();
        
        // Use the helper function to redact passwords
        let entry_modified = redact_passwords_from_entry(&mut modified_entry, &passwords);

        // Save the modified entry if it was changed
        if entry_modified {
            match sink.write().await.add_entry(&modified_entry) {
                Ok(_) => redacted_count += 1,
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, 
                        Json(json!({ "error": format!("Error saving redacted entry {}: {}", modified_entry.uuid, e) }))).into_response();
                }
            }
        }
    }

    (StatusCode::OK, Json(json!({ 
        "message": format!("Redaction complete. {} out of {} entries were modified.", redacted_count, total_entries),
        "redacted_count": redacted_count,
        "total_entries": total_entries
    }))).into_response()
}
