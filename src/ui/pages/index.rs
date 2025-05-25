use axum::{ response::Html, extract::{ State, Query } };
use urlencoding;
use crate::{ model::CaptureV2_4, sinks::Filters };
use resources::{
    CAPTURE_TYPE_EDIT_SVG,
    CAPTURE_TYPE_RUN_SVG,
    COPY_ICON_SVG,
    EDIT_ICON_SVG,
};
use crate::ui::{ resources, ProdlogUiState, highlight_matches, format_timestamp };

fn generate_index(table_rows: &str, filters: &Filters) -> String {
    let date_filter = filters.date.as_deref().unwrap_or("");
    let host_filter = filters.host.as_deref().unwrap_or("");
    let command_filter = filters.command.as_deref().unwrap_or("");
    let output_filter = filters.output.as_deref().unwrap_or("");
    let noop_filter = if filters.show_noop.unwrap_or(false) { "checked" } else { "" };
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    <link rel="stylesheet" href="/static/prodlog.css">
</head>
<body>
    <div class="container" id="container">
        <div class="header">
            <h1>Prodlog Viewer</h1>
            <button class="bluebutton" type="button" onclick="toggleWidth()" title="Toggle width">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16M4 20h16M4 12h16"/>
                </svg>
                <span class="toggle-text">Full Width</span>
            </button>
        </div>
        <div class="filters">
            <form method="get">
                <input type="date" name="date" value="{date_filter}">
                <input type="text" name="host" placeholder="Hostname" value="{host_filter}">
                <input type="text" name="command" placeholder="Command" value="{command_filter}">
                <input type="text" name="output" placeholder="Search in output" value="{output_filter}">
                    <label class="switch">
                        <input type="checkbox" name="show_noop" value="true" {noop_filter}>
                        <span class="slider"></span>
                    </label>
                    <span class="switch-label">Reveal no-op entries</span>
                <button class="bluebutton" type="submit">Filter</button>
                <button class="greybutton" type="button" onclick="window.location.href='/'">Clear</button>
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
                {table_rows}
            </tbody>
        </table>
    </div>
    <script>
        function toggleWidth() {{
            const container = document.getElementById('container');
            const toggleText = document.querySelector('.toggle-text');
            container.classList.toggle('full-width');
            toggleText.textContent = container.classList.contains('full-width') ? 'Column Width' : 'Full Width';
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
"#)
}

fn generate_entry(entry: &CaptureV2_4, filters: &Filters, preview: &Option<String>) -> String {
    let row_class = if entry.is_noop {
        " class=\"noop-row\""
    } else {
        if entry.exit_code != 0 { " class=\"error-row\"" } else { "" }
    };
    let entry_type = match entry.capture_type {
        crate::model::CaptureType::Run => CAPTURE_TYPE_RUN_SVG,
        crate::model::CaptureType::Edit => CAPTURE_TYPE_EDIT_SVG,
    };
    let start_time = format_timestamp(&entry.start_time);
    let host = entry.host.clone();
    let cmd = entry.cmd.clone();
    let copy_text = match entry.capture_type {
        crate::model::CaptureType::Run => format!("prodlog run {}", entry.cmd),
        crate::model::CaptureType::Edit => if entry.cmd.starts_with("sudo") {
            format!("prodlog edit -s {}", entry.filename)
        } else {
            format!("prodlog edit {}", entry.filename)
        }
    };
    let duration = entry.duration_ms;
    let exit_code = entry.exit_code;
    let uuid = entry.uuid.to_string();
    let link = match entry.capture_type {
        crate::model::CaptureType::Run => {
            let url = if let Some(output_filter) = &filters.output {
                if !output_filter.is_empty() {
                    format!(
                        r#"output/{}?output={}"#,
                        entry.uuid,
                        urlencoding::encode(output_filter)
                    )
                } else {
                    format!(r#"output/{}"#, entry.uuid)
                }
            } else {
                format!(r#"output/{}"#, entry.uuid)
            };
            format!(r#"<a href="{}">View</a>"#, url)
        }
        crate::model::CaptureType::Edit =>
            format!(r#"<a href="diff/{}">Diff</a>"#, entry.uuid),
    };
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
    let message_row = if !entry.message.is_empty() {
        format!(
            r#"<tr class="message-row">
                <td colspan="2"></td>
                <td colspan="6" class="message-row">
                    <div>
                        <span>{}</span>
                    </div>
                </td>
            </tr>"#,
            entry.message
        )
    } else {
        String::new()
    };
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
                        <button class="edit-or-copy-button" onclick="copyButton(this, '{copy_text}')" title="Copy">
                            {COPY_ICON_SVG}
                        </button>
                        <a href="edit/{uuid}" class="edit-or-copy-button" title="Edit command">
                            {EDIT_ICON_SVG}
                        </a>
                    </div>
                </td>
                <td>{duration}ms</td>
                <td>{exit_code}</td>
                <td>{link}{preview_html}</td>
            </tr>
            {message_row}
        </tbody>"#
    )
}

pub async fn handle_index(
    State(sink): State<ProdlogUiState>,
    Query(filters): Query<Filters>
) -> Html<String> {
    let data = match sink.read().await.get_entries(&filters) {
        Ok(data) => data,
        Err(err) => {
            return Html(String::from(format!("Error loading log data: {}", err)));
        }
    };

    let mut entries: Vec<(CaptureV2_4, Option<String>)> = if
        let Some(output_filter) = &filters.output
    {
        data.into_iter()
            .map(|entry| {
                let output_content = entry.output_as_string();
                let idx = output_content
                    .to_lowercase()
                    .find(&output_filter.to_lowercase())
                    .unwrap();
                let start = idx.saturating_sub(50);
                let end = (idx + output_filter.len() + 50).min(output_content.len());
                let preview = output_content[start..end].to_string();
                (entry, Some(preview))
            })
            .collect()
    } else {
        data.into_iter()
            .map(|entry| (entry, None))
            .collect()
    };

    entries.sort_by_key(|entry| entry.0.start_time);
    entries.reverse();

    // Generate table rows
    let rows = entries
        .iter()
        .map(|(entry, preview)| {
            generate_entry(entry, &filters, preview)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_index(&rows, &filters))
}
