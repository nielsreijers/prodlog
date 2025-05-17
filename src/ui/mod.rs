use axum::{ routing::get, Router };
use chrono::{ DateTime, Utc };
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::sinks::UiSource;

mod ansi_to_html;
mod entry;
mod index;
mod resources;
mod save;
mod static_files;

type ProdlogUiState = Arc<RwLock<Box<dyn UiSource>>>;

fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn highlight_matches(text: &str, search_term: &str) -> String {
    if search_term.is_empty() {
        return text.to_string();
    }
    text.replace(search_term, &format!("<span class=\"match-highlight\">{}</span>", search_term))
}

pub async fn run_ui(sink: Arc<RwLock<Box<dyn UiSource>>>, port: u16) {
    let app = Router::new()
        .route("/", get(index::handle_index))
        .route("/output/:uuid", get(entry::handle_output))
        .route("/diff/:uuid", get(entry::handle_diff))
        .route("/edit/:uuid", get(entry::handle_edit))
        .route("/save", axum::routing::post(save::handle_save))
        .route("/static/*path", get(static_files::serve_file))
        .with_state(sink);

    let addr = format!("0.0.0.0:{}", port);
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            // TODO: this printing could be prettier
            super::print_prodlog_message(&format!("Starting web UI on http://localhost:{}", port));
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            super::print_prodlog_message(
                &format!("Port {} is already in use. Another instance of prodlog might be running.", port)
            );
        }
        Err(e) => {
            super::print_prodlog_message(&format!("Failed to start web UI: {}", e));
        }
    }
}
