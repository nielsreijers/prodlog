use axum::{ response::Html, extract::{ State, Query, Path } };
use uuid::Uuid;
use chrono::Duration;
use similar::{ TextDiff, ChangeTag };
use html_escape;
use crate::{ model::{ CaptureType, CaptureV2_3 }, sinks::Filters };
use super::ProdlogUiState;

fn generate_entry_header(entry: &CaptureV2_3) -> String {
    let host = &entry.host;
    let cmd = &entry.cmd;
    let cwd = &entry.cwd;
    let start = super::format_timestamp(&entry.start_time);
    let end_time = entry.start_time + Duration::milliseconds(entry.duration_ms as i64);
    let end = super::format_timestamp(&end_time);
    let duration = entry.duration_ms;
    let exit = entry.exit_code;
    let message = if !entry.message.is_empty() {
        format!("<div class=\"message\">{}</div>", entry.message)
    } else {
        String::new()
    };
    let diff_or_output = if entry.capture_type == CaptureType::Edit {
        format!("<h2>{}</h2>", entry.filename)
    } else {
        format!("<h2>{}</h2>", entry.cmd)
    };
    format!(
        "
<div class=\"header-info\">
    <div class=\"info-grid\">
        <div class=\"info-item\">
            <span class=\"info-label\">Host:</span>
            <span class=\"info-value\">{host}</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">Command:</span>
            <span class=\"info-value\">{cmd}</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">Directory:</span>
            <span class=\"info-value\">{cwd}</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">Start:</span>
            <span class=\"info-value\">{start}</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">End:</span>
            <span class=\"info-value\">{end}</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">Duration:</span>
            <span class=\"info-value\">{duration}ms</span>
        </div>
        <div class=\"info-item\">
            <span class=\"info-label\">Exit Code:</span>
            <span class=\"info-value\">{exit}</span>
        </div>
    </div>
    {message}
    {diff_or_output}
</div>
    "
    )
}

fn generate_detail_page(entry: &CaptureV2_3, title: &str, content: &str) -> String {
    let header = generate_entry_header(entry);
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="/static/entry.css">
</head>
<body>
    <div class="container">
        <div class="back-link">
            <a href="/">← Back to list</a>
        </div>
        {header}
        {content}
    </div>
</body>
</html>
    "#)

}

fn generate_output_html(entry: &CaptureV2_3, output_filter: Option<&str>) -> String {
    let decoded_output = entry.output_as_string();
    let html_output = super::ansi_to_html::ansi_to_html(&decoded_output);
    let highlighted_output = if let Some(filter) = output_filter {
        super::highlight_matches(&html_output, filter)
    } else {
        html_output
    };
    let content = format!(
        r#"
        <div class="content-box">
            <pre class="command-output">{highlighted_output}</pre>
        </div>
        "#);
    generate_detail_page(entry,  "Output View", &content)
}

pub async fn handle_output(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
    Query(filters): Query<Filters>
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
        }
        Err(err) => {
            return Html(format!("Error loading log data: {}", err));
        }
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
        html.push_str(
            &format!(
                r#"<div class="{}"><span>{}</span>{}</div>"#,
                class,
                sign,
                html_escape::encode_text(change.value())
            )
        );
    }
    html
}

fn generate_diff_html(entry: &CaptureV2_3) -> String {
    if entry.capture_type != crate::model::CaptureType::Edit {
        return "Not an edit entry".to_string();
    }
    let orig = String::from_utf8_lossy(&entry.original_content);
    let edited = String::from_utf8_lossy(&entry.edited_content);
    let diff_html = simple_diff(&orig, &edited);
    let content = format!(r#"<div class="content-box">
            <pre class="command-output">{diff_html}</pre>
        </div>"#);
    generate_detail_page(entry,  "File Diff", &content)
}

pub async fn handle_diff(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>
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
        }
        Err(err) => {
            return Html(String::from(format!("Error loading log data: {}", err)));
        }
    }
}

fn generate_edit_html(entry: &CaptureV2_3) -> String {
    let message = html_escape::encode_text(&entry.message);
    let is_noop_checked = if entry.is_noop { "checked" } else { "" };
    let uuid = entry.uuid;

    let content = format!(r#"
        <form id="editForm">
            <div class="form-group">
                <label for="message">Message:</label>
                <textarea name="message" id="message" rows="10">{message}</textarea>
            </div>
            <div class="switch-container">
                <label class="switch">
                    <input type="checkbox" name="is_noop" {is_noop_checked}>
                    <span class="slider"></span>
                </label>
                <span class="switch-label">Mark as no-op (this command had no effect)</span>
            </div>
            <div class="button-group">
                <button type="submit">Save</button>
                <a href="/" class="button">Cancel</a>
            </div>
        </form>
        <script>
            document.getElementById('editForm').addEventListener('submit', async (e) => {{
                e.preventDefault();
                const form = e.target;
                const data = {{
                    uuid: '{uuid}',
                    message: form.message.value,
                    is_noop: form.is_noop.checked
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
    "#);
    generate_detail_page(entry,  "Edit Entry", &content)
}

pub async fn handle_edit(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>
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
        }
        Err(err) => {
            return Html(String::from(format!("Error loading log data: {}", err)));
        }
    }
}

