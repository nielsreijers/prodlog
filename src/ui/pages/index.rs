use axum::{ response::Html, extract::{ State, Query }, http::HeaderMap };
use crate::{ model::CaptureV2_4, sinks::Filters };
use resources::{
    CAPTURE_TYPE_EDIT_SVG,
    CAPTURE_TYPE_RUN_SVG,
    COPY_ICON_SVG,
};
use crate::ui::{ resources, ProdlogUiState, format_timestamp };

fn parse_cookies(headers: &HeaderMap) -> Filters {
    let mut filters = Filters::default();
    
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().split('=').collect();
                if parts.len() == 2 {
                    let (key, value) = (parts[0], parts[1]);
                    match key {
                        "prodlog_date" => filters.date = Some(value.to_string()),
                        "prodlog_host" => filters.host = Some(value.to_string()),
                        "prodlog_search" => filters.search = Some(value.to_string()),
                        "prodlog_show_noop" => filters.show_noop = Some(value == "true"),
                        _ => {}
                    }
                }
            }
        }
    }
    
    filters
}

fn merge_filters(url_filters: &Filters, cookie_filters: &Filters) -> Filters {
    Filters {
        date: url_filters.date.clone().or(cookie_filters.date.clone()),
        host: url_filters.host.clone().or(cookie_filters.host.clone()),
        search: url_filters.search.clone().or(cookie_filters.search.clone()),
        show_noop: url_filters.show_noop.or(cookie_filters.show_noop),
    }
}

fn generate_index(table_rows: &str, filters: &Filters) -> String {
    let date_filter = filters.date.as_deref().unwrap_or("");
    let host_filter = filters.host.as_deref().unwrap_or("");
    let search_filter = filters.search.as_deref().unwrap_or("");
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
            <form method="get" id="filterForm">
                <input type="date" name="date" id="dateFilter" value="{date_filter}">
                <input type="text" name="host" id="hostFilter" placeholder="Hostname" value="{host_filter}">
                <input type="text" name="search" id="searchFilter" placeholder="Command or message" value="{search_filter}">
                <label class="switch">
                    <input type="checkbox" name="show_noop" id="noopFilter" value="true" {noop_filter}>
                    <span class="slider"></span>
                </label>
                <span class="switch-label">Reveal no-op entries</span>
                <button class="bluebutton" type="submit">Filter</button>
                <button class="greybutton" type="button" onclick="clearFilters()">Clear</button>
            </form>
            <div class="filters-right">
                <button class="bluebutton" type="button" onclick="window.location.href='/legacy/redact'">Redact Passwords</button>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th style="width: 24px;"></th>
                    <th style="width: 190px;">Time</th>
                    <th style="width: 160px;">Host</th>
                    <th style="width: auto; white-space: normal;">Command</th>
                    <th style="width: 48px;"></th>
                    <th style="width: 80px;">Duration</th>
                    <th style="width: 30px;">Exit</th>
                </tr>
            </thead>
            <tbody>
                {table_rows}
            </tbody>
        </table>
    </div>
    <script>
        // Load saved filters from cookies on page load
        document.addEventListener('DOMContentLoaded', function() {{
            loadFilters();
        }});

        // Save filters to cookies when form is submitted
        document.getElementById('filterForm').addEventListener('submit', function() {{
            saveFilters();
        }});

        // Auto-submit form when noop filter is toggled
        document.getElementById('noopFilter').addEventListener('change', function() {{
            saveFilters();
            document.getElementById('filterForm').submit();
        }});

        function saveFilters() {{
            const filters = {{
                date: document.getElementById('dateFilter').value,
                host: document.getElementById('hostFilter').value,
                search: document.getElementById('searchFilter').value,
                show_noop: document.getElementById('noopFilter').checked
            }};
            
            // Set cookies for each filter
            if (filters.date) {{
                document.cookie = `prodlog_date=${{encodeURIComponent(filters.date)}}; path=/; max-age=86400`;
            }} else {{
                document.cookie = 'prodlog_date=; path=/; max-age=0';
            }}
            
            if (filters.host) {{
                document.cookie = `prodlog_host=${{encodeURIComponent(filters.host)}}; path=/; max-age=86400`;
            }} else {{
                document.cookie = 'prodlog_host=; path=/; max-age=0';
            }}
            
            if (filters.search) {{
                document.cookie = `prodlog_search=${{encodeURIComponent(filters.search)}}; path=/; max-age=86400`;
            }} else {{
                document.cookie = 'prodlog_search=; path=/; max-age=0';
            }}
            
            if (filters.show_noop) {{
                document.cookie = 'prodlog_show_noop=true; path=/; max-age=86400';
            }} else {{
                document.cookie = 'prodlog_show_noop=; path=/; max-age=0';
            }}
        }}

        function loadFilters() {{
            // Check if we have URL parameters first (they take precedence)
            const urlParams = new URLSearchParams(window.location.search);
            if (urlParams.toString()) {{
                return; // URL parameters exist, don't load from cookies
            }}
            
            // Load from cookies
            const cookies = document.cookie.split(';').reduce((acc, cookie) => {{
                const [key, value] = cookie.trim().split('=');
                if (key && value) {{
                    acc[key] = decodeURIComponent(value);
                }}
                return acc;
            }}, {{}});
            
            if (cookies.prodlog_date) document.getElementById('dateFilter').value = cookies.prodlog_date;
            if (cookies.prodlog_host) document.getElementById('hostFilter').value = cookies.prodlog_host;
            if (cookies.prodlog_search) document.getElementById('searchFilter').value = cookies.prodlog_search;
            if (cookies.prodlog_show_noop === 'true') document.getElementById('noopFilter').checked = true;
        }}

        function clearFilters() {{
            // Clear form fields
            document.getElementById('dateFilter').value = '';
            document.getElementById('hostFilter').value = '';
            document.getElementById('searchFilter').value = '';
            document.getElementById('noopFilter').checked = false;
            
            // Clear cookies
            document.cookie = 'prodlog_date=; path=/; max-age=0';
            document.cookie = 'prodlog_host=; path=/; max-age=0';
            document.cookie = 'prodlog_search=; path=/; max-age=0';
            document.cookie = 'prodlog_show_noop=; path=/; max-age=0';
            
            // Submit the form to refresh the page
            document.getElementById('filterForm').submit();
        }}

        function copyButton(button, text, event) {{
            event.stopPropagation(); // Prevent row click navigation
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
    }.replace("'", "\\'");
    let duration = entry.duration_ms;
    let exit_code = entry.exit_code;
    let uuid = entry.uuid.to_string();
    let message_row = if !entry.message.is_empty() {
        format!(
                                r#"<tr class="message-row clickable-row" onclick="window.location.href='/legacy/entry/{uuid}'">
                <td colspan="2"></td>
                <td colspan="5" class="message-row">
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
            <tr{row_class} class="main-row clickable-row" onclick="window.location.href='/legacy/entry/{uuid}'">
                <td>{entry_type}</td>
                <td>{start_time}</td>
                <td>{host}</td>
                <td>{cmd}</td>
                <td>
                                         <div class="button-group">
                         <button class="edit-or-copy-button" onclick="copyButton(this, '{copy_text}', event)" title="Copy">
                             {COPY_ICON_SVG}
                         </button>
                     </div>
                </td>
                <td>{duration}ms</td>
                <td>{exit_code}</td>
            </tr>
            {message_row}
        </tbody>"#
    )
}

pub async fn handle_index(
    State(sink): State<ProdlogUiState>,
    Query(url_filters): Query<Filters>,
    headers: HeaderMap,
) -> Html<String> {
    // Parse cookies and merge with URL parameters
    let cookie_filters = parse_cookies(&headers);
    let filters = merge_filters(&url_filters, &cookie_filters);

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
