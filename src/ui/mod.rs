use axum::{ routing::{get, post}, Router, response::Response, http::StatusCode };
use chrono::{ DateTime, Utc };
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::{config::get_config, sinks::UiSource};
use axum::response::Html;

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

pub async fn handle_react_app() -> Html<String> {
    // Read the React app's index.html file from embedded directory
    match static_files::get_react_index_html() {
        Some(content) => Html(content),
        None => {
            // Fallback HTML if React build is not available
            Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    <style>
        body { font-family: system-ui; padding: 2rem; text-align: center; }
        .error { color: #dc3545; }
    </style>
</head>
<body>
    <h1>Prodlog Viewer</h1>
    <div class="error">
        <p>React UI not built yet. Please run the build script:</p>
        <pre>./build-react.sh</pre>
        <p>Or use the original server-side rendered UI at: <a href="/legacy/">/legacy/</a></p>
    </div>
</body>
</html>
            "#.to_string())
        }
    }
}

pub async fn run_ui(sink: Arc<RwLock<Box<dyn UiSource>>>, port: u16) {
    let app = Router::new()
        // API routes 
        .route("/api/entries", get(rest::handle_entries_get))
        .route("/api/entries/summary", get(rest::handle_entries_summary_get))
        .route("/api/entry/:uuid", get(rest::handle_entry_get))
        .route("/api/entry", post(rest::handle_entry_post))
        .route("/api/entry/redact", post(rest::handle_entry_redact_post))
        .route("/api/redact", post(rest::handle_bulk_redact_post))  // Changed to /api/redact
        .route("/diffcontent/:uuid", get(rest::handle_diffcontent))
        
        // Legacy server-side rendered UI routes
        .route("/legacy", get(pages::index::handle_index))
        .route("/legacy/", get(pages::index::handle_index))
        .route("/legacy/redact", get(pages::redact::handle_redact_get))
        .route("/legacy/entry/:uuid", get(pages::entry::handle_entry))
        
        // Static assets
        .route("/prodlog-dyn.css", get(handle_prodlog_dyn_css))
        .route("/static/*path", get(static_files::serve_file))
        .route("/assets/*path", get(static_files::serve_react_asset))
        .route("/favicon.ico", get(static_files::serve_react_favicon))
        
        // React app SPA fallback - must be last to catch all other routes
        .fallback(get(handle_react_app))
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
