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

mod ansi_to_html;

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
        :root {{
            --proton-blue: #6D4AFF;
            --proton-blue-hover: #7B5AFF;
            --proton-background: #FFFFFF;
            --proton-text: #1C1B1F;
            --proton-text-secondary: #4E4B66;
            --proton-border: #E5E7EB;
            --proton-hover: #F5F5F5;
        }}
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 0;
            padding: 0;
            background-color: var(--proton-background);
            color: var(--proton-text);
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        h1 {{
            color: var(--proton-text);
            font-size: 2rem;
            margin-bottom: 2rem;
            font-weight: 600;
        }}
        .filters {{
            background-color: var(--proton-background);
            padding: 1.5rem;
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            margin-bottom: 2rem;
        }}
        .filters form {{
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            align-items: center;
        }}
        input, select {{
            padding: 0.75rem 1rem;
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            font-size: 0.875rem;
            color: var(--proton-text);
            background-color: var(--proton-background);
            transition: all 0.2s ease;
        }}
        input:focus, select:focus {{
            outline: none;
            border-color: var(--proton-blue);
            box-shadow: 0 0 0 2px rgba(109, 74, 255, 0.1);
        }}
        button {{
            padding: 0.75rem 1.5rem;
            background-color: var(--proton-blue);
            color: white;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.2s ease;
        }}
        button:hover {{
            background-color: var(--proton-blue-hover);
        }}
        button[type="button"] {{
            background-color: transparent;
            color: var(--proton-text);
            border: 1px solid var(--proton-border);
        }}
        button[type="button"]:hover {{
            background-color: var(--proton-hover);
        }}
        table {{
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
            margin-top: 1rem;
            background-color: var(--proton-background);
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }}
        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--proton-border);
        }}
        th {{
            background-color: var(--proton-hover);
            font-weight: 600;
            color: var(--proton-text-secondary);
        }}
        tr:hover {{
            background-color: var(--proton-hover);
        }}
        a {{
            color: var(--proton-blue);
            text-decoration: none;
            font-weight: 500;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        .output-preview {{
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            margin-top: 0.5rem;
            padding: 0.75rem;
            background-color: var(--proton-hover);
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            white-space: pre-wrap;
            max-height: 100px;
            overflow-y: auto;
            font-size: 0.875rem;
        }}
        .match-highlight {{
            background-color: rgba(109, 74, 255, 0.1);
            color: var(--proton-blue);
            padding: 0.125rem 0.25rem;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <div class="container">
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
    </div>
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
        :root {{
            --proton-blue: #6D4AFF;
            --proton-background: #1C1B1F;
            --proton-text: #FFFFFF;
            --proton-text-secondary: #A0A0A0;
            --proton-border: #2D2D2D;
        }}
        body {{ 
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            margin: 0;
            padding: 0;
            background-color: var(--proton-background);
            color: var(--proton-text);
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        pre {{ 
            white-space: pre-wrap;
            margin: 0;
            padding: 1.5rem;
            background-color: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            font-size: 0.875rem;
            line-height: 1.5;
        }}
        .back-link {{ 
            margin-bottom: 1.5rem; 
        }}
        .back-link a {{
            color: var(--proton-text-secondary);
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            font-size: 0.875rem;
            transition: color 0.2s ease;
        }}
        .back-link a:hover {{
            color: var(--proton-text);
        }}
        .match-highlight {{ 
            background-color: rgba(109, 74, 255, 0.2);
            color: var(--proton-text);
            padding: 0.125rem 0.25rem;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        <pre>{}</pre>
    </div>
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

    let html_content = ansi_to_html::ansi_to_html(&content);
    Html(generate_output_html(&html_content, filters.output.as_deref()))
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
