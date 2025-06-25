use axum::{
    response::Html,
    extract::{State, Form},
    http::StatusCode,
};
use serde::Deserialize;
use crate::{
    ui::ProdlogUiState,
    sinks::Filters,
};

#[derive(Deserialize)]
pub struct RedactForm {
    passwords: String,
}

pub async fn handle_redact_get() -> Html<String> {
    Html(generate_redact_page("", ""))
}

pub async fn handle_redact_post(
    State(sink): State<ProdlogUiState>,
    Form(form): Form<RedactForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let passwords: Vec<String> = form.passwords
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if passwords.is_empty() {
        return Ok(Html(generate_redact_page("", "No passwords provided.")));
    }

    // Get all entries
    let entries = match sink.read().await.get_entries(&Filters::default()) {
        Ok(entries) => entries,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error loading entries: {}", e))),
    };

    let mut redacted_count = 0;
    let total_entries = entries.len();

    // Process each entry
    for entry in entries {
        let mut modified_entry = entry.clone();
        let mut entry_modified = false;

        // Redact passwords in command
        for password in &passwords {
            if modified_entry.cmd.contains(password) {
                modified_entry.cmd = modified_entry.cmd.replace(password, "[REDACTED]");
                entry_modified = true;
            }
        }

        // Redact passwords in captured output
        let output_str = String::from_utf8_lossy(&modified_entry.captured_output);
        let mut new_output = output_str.to_string();
        let mut output_modified = false;
        for password in &passwords {
            if new_output.contains(password) {
                new_output = new_output.replace(password, "[REDACTED]");
                output_modified = true;
                entry_modified = true;
            }
        }
        if output_modified {
            modified_entry.captured_output = new_output.into_bytes();
        }

        // Redact passwords in original content (for edit entries)
        if !modified_entry.original_content.is_empty() {
            let original_str = String::from_utf8_lossy(&modified_entry.original_content);
            let mut new_original = original_str.to_string();
            let mut original_modified = false;
            for password in &passwords {
                if new_original.contains(password) {
                    new_original = new_original.replace(password, "[REDACTED]");
                    original_modified = true;
                    entry_modified = true;
                }
            }
            if original_modified {
                modified_entry.original_content = new_original.into_bytes();
            }
        }

        // Redact passwords in edited content (for edit entries)
        if !modified_entry.edited_content.is_empty() {
            let edited_str = String::from_utf8_lossy(&modified_entry.edited_content);
            let mut new_edited = edited_str.to_string();
            let mut edited_modified = false;
            for password in &passwords {
                if new_edited.contains(password) {
                    new_edited = new_edited.replace(password, "[REDACTED]");
                    edited_modified = true;
                    entry_modified = true;
                }
            }
            if edited_modified {
                modified_entry.edited_content = new_edited.into_bytes();
            }
        }

        // Save the modified entry if it was changed
        if entry_modified {
            match sink.write().await.add_entry(&modified_entry) {
                Ok(_) => redacted_count += 1,
                Err(e) => {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, 
                        format!("Error saving redacted entry {}: {}", modified_entry.uuid, e)));
                }
            }
        }
    }

    let message = format!("Redaction complete. {} out of {} entries were modified.", redacted_count, total_entries);
    Ok(Html(generate_redact_page(&form.passwords, &message)))
}

fn generate_redact_page(passwords: &str, message: &str) -> String {
    let message_html = if !message.is_empty() {
        format!(r#"<div class="message {}">{}</div>"#, 
            if message.contains("Error") { "error" } else { "success" }, 
            message)
    } else {
        String::new()
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Redact Passwords - Prodlog Viewer</title>
    <link rel="stylesheet" href="/prodlog-dyn.css">
    <link rel="stylesheet" href="/static/prodlog.css">
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Redact Passwords</h1>
            <button class="bluebutton" type="button" onclick="window.location.href='/'">← Back to list</button>
        </div>
        
        {message_html}
        
        <div class="section">
            <p>Enter passwords to redact from all log entries. Each password should be on a separate line.</p>
            <p><strong>Warning:</strong> This operation will permanently modify your log data. Make sure you have a backup.</p>
            
            <form method="post" action="/redact" onsubmit="return confirmRedaction()">
                <div class="form-group">
                    <label for="passwords">Passwords to redact (one per line):</label>
                    <textarea id="passwords" name="passwords" rows="10" cols="50" placeholder="password123&#10;secret456&#10;mytoken789">{passwords}</textarea>
                </div>
                <div class="form-group">
                    <button class="redbutton" type="submit">Redact Passwords</button>
                    <button class="greybutton" type="button" onclick="document.getElementById('passwords').value = ''">Clear</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        function confirmRedaction() {{
            const passwords = document.getElementById('passwords').value.trim();
            if (!passwords) {{
                alert('Please enter at least one password to redact.');
                return false;
            }}
            
            const passwordCount = passwords.split('\\n').filter(line => line.trim()).length;
            const message = `Are you sure you want to redact ${{passwordCount}} password(s) from all log entries? This operation will permanently modify your log data and cannot be undone.`;
            
            return confirm(message);
        }}
    </script>
</body>
</html>
"#)
} 