use axum::{ response::Html, extract::State, Json };
use uuid::Uuid;
use serde::Deserialize;

use super::ProdlogUiState;

#[derive(Deserialize)]
pub struct SaveRequest {
    pub uuid: String,
    pub message: String,
    pub is_noop: bool,
}

pub async fn handle_save(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<SaveRequest>
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
