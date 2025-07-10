use axum::{ routing::{get, post}, Router, response::Response, http::StatusCode };
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::{config::get_config, sinks::UiSource};
use axum::response::Html;

mod rest;
mod static_files;

type ProdlogUiState = Arc<RwLock<Box<dyn UiSource>>>;

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
        
        // Task management routes
        .route("/api/tasks", get(rest::handle_tasks_get))
        .route("/api/task", post(rest::handle_task_create_post))
        .route("/api/task/update", post(rest::handle_task_update_post))
        .route("/api/entries/ungroup", post(rest::handle_entries_ungroup_post))
        

        
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
