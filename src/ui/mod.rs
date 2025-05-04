use axum::{
    routing::get,
    Router,
    response::Html,
    extract::{State, Query, Path},
};
use serde::Deserialize;
use uuid::Uuid;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use urlencoding;

use crate::model::CaptureV2_2;

mod ansi_to_html;

fn load_log_data(json_path: &PathBuf) -> Result<Vec<CaptureV2_2>, std::io::Error> {
    let data = crate::sinks::json::read_prodlog_data(json_path)?;
    Ok(data.entries)
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
            transition: max-width 0.3s ease;
        }}
        .container.full-width {{
            max-width: none;
            padding: 2rem;
        }}
        .header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 2rem;
        }}
        .view-toggle {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.75rem 1.5rem;
            background-color: var(--proton-blue);
            color: white;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.2s ease;
        }}
        .view-toggle:hover {{
            background-color: var(--proton-blue-hover);
        }}
        .view-toggle svg {{
            width: 16px;
            height: 16px;
            stroke: currentColor;
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
        tr.error-row {{
            background-color: #ffeaea !important;
        }}
    </style>
</head>
<body>
    <div class="container" id="container">
        <div class="header">
            <h1>Prodlog Viewer</h1>
            <button class="view-toggle" onclick="toggleWidth()" title="Toggle width">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16M4 20h16M4 12h16"/>
                </svg>
                <span class="toggle-text">Full Width</span>
            </button>
        </div>
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
                    <th>Exit</th>
                    <th>Log</th>
                </tr>
            </thead>
            <tbody>
                {4}
            </tbody>
        </table>
    </div>
    <script>
        function toggleWidth() {{
            const container = document.getElementById('container');
            const toggleText = document.querySelector('.toggle-text');
            container.classList.toggle('full-width');
            toggleText.textContent = container.classList.contains('full-width') ? 'Column Width' : 'Full Width';
            // Store preference in localStorage
            localStorage.setItem('fullWidth', container.classList.contains('full-width'));
        }}
        
        // Restore preference on page load
        document.addEventListener('DOMContentLoaded', () => {{
            const container = document.getElementById('container');
            const toggleText = document.querySelector('.toggle-text');
            if (localStorage.getItem('fullWidth') === 'true') {{
                container.classList.add('full-width');
                toggleText.textContent = 'Column Width';
            }}
        }});
    </script>
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

fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn output_to_string(output: Vec<u8>) -> String {
    String::from_utf8(output.clone()).unwrap()
}

async fn index(
    State(state): State<Arc<PathBuf>>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    // Read the JSON file
    let json_path = state.join("prodlog.json");
    let data = match load_log_data(&json_path) {
        Ok(data) => data,
        Err(err) => return Html(String::from(format!("Error loading log data: {}", err))),
    };

    // Filter entries
    let mut filtered_entries = Vec::new();
    
    for entry in &data {
        // Apply date, host, and command filters
        if let Some(date) = &filters.date {
            if !entry.start_time.to_rfc3339().starts_with(date) {
                continue;
            }
        }
        
        if let Some(host) = &filters.host {
            if !entry.host.to_lowercase().contains(&host.to_lowercase()) {
                continue;
            }
        }
        
        if let Some(command) = &filters.command {
            if !entry.cmd.to_lowercase().contains(&command.to_lowercase()) {
                continue;
            }
        }

        // Check output content if output filter is present
        if let Some(output_filter) = &filters.output {
            if !output_filter.is_empty() {
                let output_content = output_to_string(entry.captured_output.clone());
                if !output_content.to_lowercase().contains(&output_filter.to_lowercase()) {
                    continue;
                }
                // Add preview of matching output_content
                let idx = output_content.to_lowercase().find(&output_filter.to_lowercase()).unwrap();
                let start = idx.saturating_sub(50);
                let end = (idx + output_filter.len() + 50).min(output_content.len());
                let preview = output_content[start..end].to_string();
                filtered_entries.push((entry, Some(preview)));
                continue;
            }
        }
        
        filtered_entries.push((entry, None));
    }

    filtered_entries.sort_by_key(| entry | &entry.0.start_time);
    filtered_entries.reverse();

    // Generate table rows
    let rows = filtered_entries.iter()
        .map(|(entry, preview)| {
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
            let row_class = if entry.exit_code != 0 { " class=\"error-row\"" } else { "" };
            let output_link = if let Some(output_filter) = &filters.output {
                if !output_filter.is_empty() {
                    format!(r#"output/{}?output={}"#, entry.uuid, urlencoding::encode(output_filter))
                } else {
                    format!(r#"output/{}"#, entry.uuid)
                }
            } else {
                format!(r#"output/{}"#, entry.uuid)
            };
            format!(
                r#"<tr{}>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}ms</td>
                    <td>{}</td>
                    <td>
                        <a href="{}">View</a>
                        {}
                    </td>
                </tr>"#,
                row_class,
                format_timestamp(&entry.start_time),
                entry.host,
                entry.cmd,
                entry.duration_ms,
                entry.exit_code,
                output_link,
                preview_html
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_html(&rows, &filters))
}

fn generate_output_html(entry: &CaptureV2_2, output_filter: Option<&str>) -> String {
    // Format times
    let start = format_timestamp(&entry.start_time);
    let end_time = entry.start_time + Duration::milliseconds(entry.duration_ms as i64);
    let end =  format_timestamp(&end_time);
    let duration = entry.duration_ms;
    let exit = entry.exit_code;
    let host = &entry.host;
    let command = &entry.cmd;

    let decoded_output = output_to_string(entry.captured_output.clone());
    let html_output = ansi_to_html::ansi_to_html(&decoded_output);
    let highlighted_output = if let Some(filter) = output_filter {
        highlight_matches(&html_output, filter)
    } else {
        html_output
    };

    format!(
r#"<!DOCTYPE html>
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
        .command-output {{ 
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
            background-color: #ffeb3b;
            color: #222;
            padding: 2px 4px;
            border-radius: 4px;
            font-weight: bold;
            box-shadow: 0 0 0 2px #fff59d;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        <pre>
        Host:     {host}
        Command:  {command}
        Start:    {start}
        End:      {end}
        Duration: {duration}ms
        ExitCode: {exit}
        Output:
        </pre>
        <pre class="command-output">{highlighted_output}</pre>
    </div>
</body>
</html>
    "#)
}

async fn view_output(
    State(state): State<Arc<PathBuf>>,
    Path(uuid): Path<String>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    let json_path = state.join("prodlog.json");
    let data = match load_log_data(&json_path) {
        Ok(data) => data,
        Err(err) => return Html(String::from(format!("Error loading log data: {}", err))),
    };

    // Find the entry with the matching uuid
    let uuid = Uuid::parse_str(&uuid).unwrap();
    let entry = data.iter().find(|e| e.uuid == uuid);
    if let Some(entry) = entry {
        Html(generate_output_html(entry, filters.output.as_deref()))
    } else {
        Html(String::from("Entry not found"))
    }
}

pub async fn run_ui(log_dir: &PathBuf, port: u16) {
    let app_state = Arc::new(log_dir.clone()); 
   
    let app = Router::new()
        .route("/", get(index))
        .route("/output/:uuid", get(view_output))
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);    
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            // TODO: this printing could be prettier
            super::print_prodlog_message(&format!("Starting web UI on http://localhost:{}", port));
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            super::print_prodlog_message(&format!("Port {} is already in use. Another instance of prodlog might be running.", port));
        }
        Err(e) => {
            super::print_prodlog_message(&format!("Failed to start web UI: {}", e));
        }
    }
}
