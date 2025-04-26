use axum::{
    routing::get,
    Router,
    response::Html,
    extract::State,
    serve,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    start_time: String,
    host: String,
    command: String,
    duration_ms: u64,
    log_filename: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogData {
    entries: Vec<LogEntry>,
}

fn generate_html(table_rows: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        table {{ 
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }}
        th, td {{ 
            padding: 8px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{ background-color: #f5f5f5; }}
    </style>
</head>
<body>
    <h1>Prodlog Viewer</h1>
    <table>
        <thead>
            <tr>
                <th>Time</th>
                <th>Host</th>
                <th>Command</th>
                <th>Duration</th>
                <th>Log</th>
            </tr>
        </thead>
        <tbody>
            {0}
        </tbody>
    </table>
</body>
</html>
"#, table_rows)
}

async fn index(State(state): State<Arc<PathBuf>>) -> Html<String> {
    // Read the JSON file
    let json_path = state.join("prodlog.json");
    let json_content = match fs::read_to_string(json_path).await {
        Ok(content) => content,
        Err(_) => return Html(generate_html(""))
    };

    // Parse JSON
    let data: LogData = match serde_json::from_str(&json_content) {
        Ok(data) => data,
        Err(_) => return Html(generate_html(""))
    };

    // Generate table rows
    let rows = data.entries.iter()
        .map(|entry| format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}ms</td>
                <td><a href="/output/{}">View</a></td>
            </tr>"#,
            entry.start_time,
            entry.host,
            entry.command,
            entry.duration_ms,
            entry.log_filename
        ))
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_html(&rows))
}

pub async fn run_ui(log_dir: &PathBuf) {
    let app_state = Arc::new(log_dir.clone());
    
    let app = Router::new()
        .route("/", get(index))
        .with_state(app_state);

    println!("Starting web UI on http://localhost:3000");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
