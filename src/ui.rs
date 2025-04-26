use axum::{
    routing::get,
    Router,
    response::Html,
    extract::{State, Query, Path},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use urlencoding;

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
        .output-preview {{
            font-family: monospace;
            margin-top: 5px;
            padding: 5px;
            background-color: #f8f8f8;
            border: 1px solid #ddd;
            border-radius: 4px;
            white-space: pre-wrap;
            max-height: 100px;
            overflow-y: auto;
        }}
        .match-highlight {{
            background-color: #fff3cd;
            padding: 2px;
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

fn highlight_matches(text: &str, search_term: &str) -> String {
    if search_term.is_empty() {
        return text.to_string();
    }
    text.replace(search_term, &format!("<span class=\"match-highlight\">{}</span>", search_term))
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
    let mut filtered_entries = Vec::new();
    
    for entry in &data.entries {
        // Apply date, host, and command filters
        if let Some(date) = &filters.date {
            if !entry.start_time.starts_with(date) {
                continue;
            }
        }
        
        if let Some(host) = &filters.host {
            if !entry.host.to_lowercase().contains(&host.to_lowercase()) {
                continue;
            }
        }
        
        if let Some(command) = &filters.command {
            if !entry.command.to_lowercase().contains(&command.to_lowercase()) {
                continue;
            }
        }

        // Check output content if output filter is present
        if let Some(output_filter) = &filters.output {
            if !output_filter.is_empty() {
                let log_path = state.join(&entry.log_filename);
                if let Ok(content) = fs::read_to_string(log_path).await {
                    if !content.to_lowercase().contains(&output_filter.to_lowercase()) {
                        continue;
                    }
                    // Add preview of matching content
                    let idx = content.to_lowercase().find(&output_filter.to_lowercase()).unwrap();
                    let start = idx.saturating_sub(50);
                    let end = (idx + output_filter.len() + 50).min(content.len());
                    let preview = content[start..end].to_string();
                    filtered_entries.push((entry, Some(preview)));
                    continue;
                }
            }
        }
        
        filtered_entries.push((entry, None));
    }

    // Generate table rows
    let rows = filtered_entries.iter()
        .map(|(entry, preview)| {
            let encoded_path = urlencoding::encode(&entry.log_filename);
            let preview_html = if let Some(preview) = preview {
                if let Some(output_filter) = &filters.output {
                    format!(
                        r#"<div class="output-preview">{}</div>"#,
                        highlight_matches(preview, output_filter)
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}ms</td>
                    <td>
                        <a href="/output/{}">View</a>
                        {}
                    </td>
                </tr>"#,
                entry.start_time,
                entry.host,
                entry.command,
                entry.duration_ms,
                encoded_path,
                preview_html
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_html(&rows, &filters))
}

fn generate_output_html(content: &str, output_filter: Option<&str>) -> String {
    let highlighted_content = if let Some(filter) = output_filter {
        highlight_matches(content, filter)
    } else {
        content.to_string()
    };

    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Output View</title>
    <style>
        body {{ 
            font-family: monospace; 
            margin: 20px; 
            background-color: #1e1e1e;
            color: #d4d4d4;
        }}
        pre {{ 
            white-space: pre-wrap;
            margin: 0;
            padding: 10px;
        }}
        .back-link {{ 
            margin-bottom: 20px; 
        }}
        .back-link a {{
            color: #4CAF50;
            text-decoration: none;
        }}
        .back-link a:hover {{
            text-decoration: underline;
        }}
        .match-highlight {{ 
            background-color: #fff3cd; 
            color: #000;
            padding: 2px; 
        }}
    </style>
</head>
<body>
    <div class="back-link">
        <a href="/">← Back to list</a>
    </div>
    <pre>{}</pre>
</body>
</html>
    "#, highlighted_content)
}

async fn view_output(
    State(state): State<Arc<PathBuf>>,
    Path(filepath): Path<String>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    // URL decode the filepath
    let decoded_path = urlencoding::decode(&filepath)
        .unwrap_or(std::borrow::Cow::from(&filepath))
        .into_owned();
    
    let file_path = state.join(decoded_path);
    
    // Security check to prevent directory traversal
    if !file_path.starts_with(&*state) {
        return Html(String::from("Access denied"));
    }
    
    let content = match fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(_) => String::from("File not found"),
    };

    Html(generate_output_html(&content, filters.output.as_deref()))
}

pub async fn run_ui(log_dir: &PathBuf) {
    let app_state = Arc::new(log_dir.clone()); 
   
    let app = Router::new()
        .route("/", get(index))
        .route("/output/:filepath", get(view_output))
        .with_state(app_state);

    println!("Starting web UI on http://localhost:3000");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
