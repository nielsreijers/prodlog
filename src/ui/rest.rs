use axum::{ extract::{Path, State}, http::StatusCode, response::{Html, IntoResponse}, Json };
use serde::Deserialize;
use serde_json::json;
use similar::{ ChangeTag, TextDiff };
use uuid::Uuid;

use super::ProdlogUiState;

#[derive(Deserialize)]
pub struct EntryPostData {
    pub uuid: String,
    pub message: String,
    pub is_noop: bool,
}

pub async fn handle_entry_post(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<EntryPostData>
) -> Html<String> {
    let uuid = match Uuid::parse_str(&data.uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Html(String::from("Invalid UUID"));
        }
    };

    let entry = match sink.read().await.get_entry_by_id(uuid) {
        Ok(Some(mut entry)) => {
            entry.message = data.message;
            entry.is_noop = data.is_noop;
            entry
        }
        _ => {
            return Html(String::from("Entry not found"));
        }
    };

    match sink.write().await.add_entry(&entry) {
        Ok(_) => Html(String::from("Success")),
        Err(err) => Html(format!("Error saving entry: {}", err)),
    }
}

pub async fn handle_entry_get(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid UUID format"
                }))
            ).into_response();
        }
    };

    let entry = match sink.read().await.get_entry_by_id(uuid) {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Entry not found"
                }))
            ).into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Error loading entry: {}", err)
                }))
            ).into_response();
        }
    };

    (StatusCode::OK, Json(entry)).into_response()
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
    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid UUID format"
                }))
            ).into_response();
        }
    };

    let entry = match sink.read().await.get_entry_by_id(uuid) {
        Ok(Some(entry)) => {
            if entry.capture_type != crate::model::CaptureType::Edit {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "Not an edit entry"
                    }))
                ).into_response();
            } else {
                entry
            }        
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Entry not found"
                }))
            ).into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Error loading entry: {}", err)
                }))
            ).into_response();
        }
    };
    let orig = String::from_utf8_lossy(&entry.original_content);
    let edited = String::from_utf8_lossy(&entry.edited_content);
    (StatusCode::OK, Json(json!({ "diff": simple_diff(&orig, &edited) }))).into_response()
}

