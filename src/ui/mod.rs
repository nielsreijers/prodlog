use axum::{
    routing::get,
    Router,
    response::Html,
    extract::{State, Query, Path},
    Json,
};
use tokio::sync::RwLock;
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use urlencoding;
use similar::{TextDiff, ChangeTag};
use html_escape;
use crate::{model::{CaptureType, CaptureV2_2}, sinks::{Filters, UiSource}};
use resources::{CAPTURE_TYPE_EDIT_SVG, CAPTURE_TYPE_RUN_SVG, COPY_ICON_SVG, EDIT_ICON_SVG, MAIN_CSS, OUTPUT_CSS};
use serde::{Deserialize};

mod ansi_to_html;
mod resources;

type ProdlogUiState = Arc<RwLock<Box<dyn UiSource>>>;

#[derive(Deserialize)]
struct SaveRequest {
    uuid: String,
    message: String,
}

fn generate_html(table_rows: &str, filters: &Filters) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    {MAIN_CSS}
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
                    <th style="width: 24px;"></th>
                    <th style="width: 190px;">Time</th>
                    <th style="width: 120px;">Host</th>
                    <th style="width: auto; white-space: normal;">Command</th>
                    <th style="width: 48px;"></th>
                    <th style="width: 80px;">Duration</th>
                    <th style="width: 30px;">Exit</th>
                    <th style="width: 50px;">Log</th>
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
        
        function copyButton(button, text) {{
            navigator.clipboard.writeText(text).then(() => {{
                button.classList.add('copied');
                button.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6L9 17l-5-5"/></svg>';
                setTimeout(() => {{
                    button.classList.remove('copied');
                    button.innerHTML = `{COPY_ICON_SVG}`;
                }}, 2000);
            }});
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

async fn index(
    State(sink): State<ProdlogUiState>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    let data = match sink.read().await.get_entries(&filters) {
        Ok(data) => data,
        Err(err) => return Html(String::from(format!("Error loading log data: {}", err))),
    };

    let mut entries: Vec<(CaptureV2_2, Option<String>)> = if let Some(output_filter) = &filters.output {
        if !output_filter.is_empty() {
            data.into_iter().map(|entry| {
                let output_content = entry.output_as_string();
                let idx = output_content.to_lowercase().find(&output_filter.to_lowercase()).unwrap();
                let start = idx.saturating_sub(50);
                let end = (idx + output_filter.len() + 50).min(output_content.len());
                let preview = output_content[start..end].to_string();
                (entry, Some(preview)
            )}).collect()            
        } else {
            data.into_iter().map(|entry| (entry, None)).collect()
        }
    } else {
        data.into_iter().map(|entry| (entry, None)).collect()
    };
    
    entries.sort_by_key(| entry | entry.0.start_time);
    entries.reverse();

    // Generate table rows
    let rows = entries.iter()
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
            let link = match entry.capture_type {
                crate::model::CaptureType::Run => {
                    let url =if let Some(output_filter) = &filters.output {
                        if !output_filter.is_empty() {
                            format!(r#"output/{}?output={}"#, entry.uuid, urlencoding::encode(output_filter))
                        } else {
                            format!(r#"output/{}"#, entry.uuid)
                        }
                    } else {
                        format!(r#"output/{}"#, entry.uuid)
                    };
                    format!(r#"<a href="{}">View</a>"#, url)
                },
                crate::model::CaptureType::Edit => format!(r#"<a href="diff/{}">Diff</a>"#, entry.uuid),
            };
            let entry_type = match entry.capture_type {
                crate::model::CaptureType::Run => CAPTURE_TYPE_RUN_SVG,
                crate::model::CaptureType::Edit => CAPTURE_TYPE_EDIT_SVG,
            };
            let copy_text = match entry.capture_type {
                crate::model::CaptureType::Run => format!("prodlog run {}", entry.cmd),
                crate::model::CaptureType::Edit => if entry.cmd.starts_with("sudo") {
                    format!("prodlog edit -s {}", entry.filename)
                } else {
                    format!("prodlog edit {}", entry.filename)                    
                }
            };
            let message_row = if !entry.message.is_empty() {
                format!(
                    r#"<tr class="message-row">
                        <td colspan="2"></td>
                        <td colspan="6" class="message">
                            <div class="message-content">
                                <span>{}</span>
                            </div>
                        </td>
                    </tr>"#,
                    entry.message
                )
            } else {
                String::new()
            };
            let uuid = entry.uuid.to_string();
            let start_time = format_timestamp(&entry.start_time);
            let host = entry.host.clone();
            let cmd = entry.cmd.clone();
            let duration = entry.duration_ms;
            let exit_code = entry.exit_code;
            format!(
                r#"
                <tbody>
                    <tr{row_class} class="main-row">
                        <td>{entry_type}</td>
                        <td>{start_time}</td>
                        <td>{host}</td>
                        <td>{cmd}</td>
                        <td>
                            <div class="button-group">
                                <button class="copy-button" onclick="copyButton(this, '{copy_text}')" title="Copy">
                                    {COPY_ICON_SVG}
                                </button>
                                <a href="edit/{uuid}" class="copy-button" title="Edit command">
                                    {EDIT_ICON_SVG}
                                </a>
                            </div>
                        </td>
                        <td>{duration}ms</td>
                        <td>{exit_code}</td>
                        <td>{link}{preview_html}</td>
                    </tr>
                    {message_row}
                </tbody>"#,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_html(&rows, &filters))
}

fn generate_entry_header(entry: &CaptureV2_2) -> String {
    let host = &entry.host;
    let cmd = &entry.cmd;
    let cwd = &entry.cwd;
    let start = format_timestamp(&entry.start_time);
    let end_time = entry.start_time + Duration::milliseconds(entry.duration_ms as i64);
    let end =  format_timestamp(&end_time);
    let duration = entry.duration_ms;
    let exit = entry.exit_code;
    let message = if !entry.message.is_empty() {
        format!("Message:   {}\n", entry.message)
    } else {
        String::new()
    };
    let diff_or_output = if entry.capture_type == CaptureType::Edit {
        format!("<h2>{}</h2>", entry.filename)
    } else {
        format!("<h2>{}</h2>", entry.cmd)
    };
    format!("
<pre>
Host:      {host}
Command:   {cmd}
Directory: {cwd}
Start:     {start}
End:       {end}
Duration:  {duration}ms
ExitCode:  {exit}
{message}
{diff_or_output}
</pre>
    ")
}



fn generate_output_html(entry: &CaptureV2_2, output_filter: Option<&str>) -> String {
    let header = generate_entry_header(entry);
    let decoded_output = entry.output_as_string();
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
    {OUTPUT_CSS}
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        {header}
        <pre class="command-output">{highlighted_output}</pre>
    </div>
</body>
</html>
    "#)
}

async fn view_output(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
    Query(filters): Query<Filters>,
) -> Html<String> {
    let uuid = Uuid::parse_str(&uuid).unwrap();
    let entry = sink.read().await.get_entry_by_id(uuid);

    match entry {
        Ok(entry) => {
            if let Some(entry) = entry {
                Html(generate_output_html(&entry, filters.output.as_deref()))
            } else {
                Html(String::from("Entry not found"))
            }        
        },
        Err(err) => return Html(format!("Error loading log data: {}", err)),
    }
}

fn simple_diff(orig: &str, edited: &str) -> String {
    let diff = TextDiff::from_lines(orig, edited);
    let mut html = String::new();
    for change in diff.iter_all_changes() {
        let (class, sign) = match change.tag() {
            ChangeTag::Delete => ("diff-del", "-"),
            ChangeTag::Insert => ("diff-ins", "+"),
            ChangeTag::Equal => ("", " "),
        };
        html.push_str(&format!(
            r#"<div class="{}"><span>{}</span>{}</div>"#,
            class, sign, html_escape::encode_text(change.value())
        ));
    }
    html
}

fn generate_diff_html(entry: &CaptureV2_2) -> String {
    if entry.capture_type != crate::model::CaptureType::Edit {
        return "Not an edit entry".to_string();
    }
    let header = generate_entry_header(entry);
    let orig = String::from_utf8_lossy(&entry.original_content);
    let edited = String::from_utf8_lossy(&entry.edited_content);
    let diff_html = simple_diff(&orig, &edited);
    format!(
r#"<!DOCTYPE html>
<html>
<head>
    <title>File Diff</title>
    {OUTPUT_CSS}
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        {header}
        <pre class="command-output">{diff_html}</pre>
    </div>
</body>
</html>
"#)
}

async fn view_diff(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> Html<String> {
    let uuid = Uuid::parse_str(&uuid).unwrap();
    let entry = sink.read().await.get_entry_by_id(uuid);

    match entry {
        Ok(entry) => {
            if let Some(entry) = entry {
                Html(generate_diff_html(&entry))
            } else {
                Html(String::from("Entry not found"))
            }        
        },
        Err(err) => return Html(String::from(format!("Error loading log data: {}", err))),
    }
}

fn generate_edit_html(entry: &CaptureV2_2) -> String {
    let header = generate_entry_header(entry);

    format!(
r#"<!DOCTYPE html>
<html>
<head>
    <title>Edit Entry</title>
    {OUTPUT_CSS}
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        {header}
        <form id="editForm">
            <i>Message:</i>
            <textarea name="message" rows="10" style="width: 100%; margin: 1rem 0;">{message}</textarea>
            <div class="button-group">
                <button type="submit">Save</button>
                <a href="/" class="button">Cancel</a>
            </div>
        </form>
    </div>
    <script>
        document.getElementById('editForm').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const form = e.target;
            const data = {{
                uuid: '{uuid}',
                message: form.message.value
            }};
            try {{
                const response = await fetch('/save', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify(data)
                }});
                if (response.ok) {{
                    window.location.href = '/';
                }} else {{
                    alert('Failed to save changes');
                }}
            }} catch (error) {{
                alert('Error saving changes: ' + error);
            }}
        }});
    </script>
</body>
</html>
"#,
        uuid = entry.uuid,
        message = html_escape::encode_text(&entry.message)
    )
}

async fn view_edit(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> Html<String> {
    let uuid = Uuid::parse_str(&uuid).unwrap();
    let entry = sink.read().await.get_entry_by_id(uuid);

    match entry {
        Ok(entry) => {
            if let Some(entry) = entry {
                Html(generate_edit_html(&entry))
            } else {
                Html(String::from("Entry not found"))
            }        
        },
        Err(err) => return Html(String::from(format!("Error loading log data: {}", err))),
    }
}

async fn save_entry(
    State(sink): State<ProdlogUiState>,
    Json(data): Json<SaveRequest>,
) -> Html<String> {
    let uuid = match Uuid::parse_str(&data.uuid) {
        Ok(uuid) => uuid,
        Err(_) => return Html(String::from("Invalid UUID")),
    };

    let entry = match sink.read().await.get_entry_by_id(uuid) {
        Ok(Some(mut entry)) => {
            entry.message = data.message;
            entry
        },
        _ => return Html(String::from("Entry not found")),
    };

    match sink.write().await.add_entry(&entry) {
        Ok(_) => Html(String::from("Success")),
        Err(err) => Html(format!("Error saving entry: {}", err)),
    }
}

pub async fn run_ui(sink: Arc<RwLock<Box<dyn UiSource>>>, port: u16) {
    let app = Router::new()
        .route("/", get(index))
        .route("/output/:uuid", get(view_output))
        .route("/diff/:uuid", get(view_diff))
        .route("/edit/:uuid", get(view_edit))
        .route("/save", axum::routing::post(save_entry))
        .with_state(sink);

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
