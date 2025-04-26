use axum::{
    routing::get,
    Router,
    response::Html,
    extract::{State, Query},
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

// Add query parameters struct for filters
#[derive(Deserialize, Debug, Default)]
struct Filters {
    date: Option<String>,
    host: Option<String>,
    command: Option<String>,
    output: Option<String>,
}

fn generate_html(table_rows: &str, filters: &Filters) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .filters {{ margin-bottom: 20px; }}
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
        input, select {{ 
            padding: 5px;
            margin-right: 10px;
        }}
        button {{
            padding: 5px 10px;
            background-color: #4CAF50;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }}
        button:hover {{
            background-color: #45a049;
        }}
    </style>
</head>
<body>
    <h1>Prodlog Viewer</h1>
    <div class="filters">
        <form method="get">
            <input type="date" name="date" value="{0}">
            <input type="text" name="host" placeholder="Hostname" value="{1}">
            <input type="text" name="command" placeholder="Command" value="{2}">
            <input type="text" name="output" placeholder="Search in output" value="{3}">
            <button type="submit">Filter</button>
            <button type="button" onclick="window.location.href='/'">Clear</button>
        </form>
    </div>
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
            {4}
        </tbody>
    </table>
</body>
</html>
"#, 
        filters.date.as_deref().unwrap_or(""),
        filters.host.as_deref().unwrap_or(""),
        filters.command.as_deref().unwrap_or(""),
        filters.output.as_deref().unwrap_or(""),
        table_rows
    )
}

async fn index(
    State(state): State<Arc<PathBuf>>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    // Read the JSON file
    let json_path = state.join("prodlog.json");
    let json_content = match fs::read_to_string(json_path).await {
        Ok(content) => content,
        Err(_) => return Html(generate_html("", &filters))
    };

    // Parse JSON
    let data: LogData = match serde_json::from_str(&json_content) {
        Ok(data) => data,
        Err(_) => return Html(generate_html("", &filters))
    };

    // Filter entries
    let filtered_entries: Vec<&LogEntry> = data.entries.iter()
        .filter(|entry| {
            // Date filter
            if let Some(date) = &filters.date {
                if !entry.start_time.starts_with(date) {
                    return false;
                }
            }
            
            // Host filter
            if let Some(host) = &filters.host {
                if !entry.host.to_lowercase().contains(&host.to_lowercase()) {
                    return false;
                }
            }
            
            // Command filter
            if let Some(command) = &filters.command {
                if !entry.command.to_lowercase().contains(&command.to_lowercase()) {
                    return false;
                }
            }

            // Output filter would go here, but it requires reading the log files
            // We can add that later if needed

            true
        })
        .collect();

    // Generate table rows
    let rows = filtered_entries.iter()
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

    Html(generate_html(&rows, &filters))
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
