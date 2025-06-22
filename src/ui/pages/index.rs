use axum::{ response::Html, extract::{ State, Query } };
use crate::{ model::CaptureV2_4, sinks::Filters };
use resources::{
    CAPTURE_TYPE_EDIT_SVG,
    CAPTURE_TYPE_RUN_SVG,
    COPY_ICON_SVG,
    EDIT_ICON_SVG,
};
use crate::ui::{ resources, ProdlogUiState, format_timestamp };

fn generate_index(table_rows: &str, filters: &Filters) -> String {
    let date_filter = filters.date.as_deref().unwrap_or("");
    let host_filter = filters.host.as_deref().unwrap_or("");
    let command_filter = filters.command.as_deref().unwrap_or("");
    let noop_filter = if filters.show_noop.unwrap_or(false) { "checked" } else { "" };
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Prodlog Viewer</title>
    <link rel="stylesheet" href="/prodlog-dyn.css">
    <link rel="stylesheet" href="/static/prodlog.css">
</head>
<body>
    <div class="container" id="container">
        <div class="header">
            <h1>Prodlog Viewer</h1>
        </div>
        <div class="filters">
            <form method="get">
                <input type="date" name="date" value="{date_filter}">
                <input type="text" name="host" placeholder="Hostname" value="{host_filter}">
                <input type="text" name="command" placeholder="Command" value="{command_filter}">
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
    </script>
</body>
</html>
"#)
}

fn generate_entry(entry: &CaptureV2_4) -> String {
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
        crate::model::CaptureType::Run =>
            format!(r#"<a href="output/{}">View</a>"#, entry.uuid),
        crate::model::CaptureType::Edit =>
            format!(r#"<a href="diff/{}">Diff</a>"#, entry.uuid),
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
                <td>{link}</td>
            </tr>
            {message_row}
        </tbody>"#
    )
}

pub async fn handle_index(
    State(sink): State<ProdlogUiState>,
    Query(filters): Query<Filters>
) -> Html<String> {
    let mut entries = match sink.read().await.get_entries(&filters) {
        Ok(data) => data,
        Err(err) => {
            return Html(String::from(format!("Error loading log data: {}", err)));
        }
    };

    entries.sort_by_key(|entry| entry.start_time);
    entries.reverse();

    // Generate table rows
    let rows = entries
        .iter()
        .map(|entry| generate_entry(entry))
        .collect::<Vec<_>>()
        .join("\n");

    Html(generate_index(&rows, &filters))
}
