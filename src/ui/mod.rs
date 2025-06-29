use axum::{ routing::{get, post}, Router, response::Response, http::StatusCode };
use chrono::{ DateTime, Utc };
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::{config::get_config, sinks::UiSource};

mod resources;
mod rest;
mod pages;
mod static_files;

type ProdlogUiState = Arc<RwLock<Box<dyn UiSource>>>;

fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub async fn handle_prodlog_dyn_css() -> Response {
    let background = get_config().ui_background.clone();
    let css = format!("
        :root {{
            --prodlog-dyn-background: {background};
        }}
    ");
    
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/css")
        .body(axum::body::Body::from(css))
        .unwrap()
}

pub async fn run_ui(sink: Arc<RwLock<Box<dyn UiSource>>>, port: u16) {
    let app = Router::new()
        .route("/", get(pages::index::handle_index))
        .route("/prodlog-dyn.css", get(handle_prodlog_dyn_css))
        .route("/redact", get(pages::redact::handle_redact_get))
        .route("/redact", post(rest::handle_bulk_redact_post))
        .route("/output/:uuid", get(pages::entry::output::handle_output))
        .route("/diff/:uuid", get(pages::entry::diff::handle_diff))
        .route("/diffcontent/:uuid", get(rest::handle_diffcontent))
        .route("/edit/:uuid", get(pages::entry::edit::handle_edit))
        .route("/entry/:uuid", get(rest::handle_entry_get))
        .route("/entry", post(rest::handle_entry_post))
        .route("/entry/redact", post(rest::handle_entry_redact_post))
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
