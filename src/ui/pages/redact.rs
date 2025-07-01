use axum::{
    response::Html,
};

pub async fn handle_redact_get() -> Html<String> {
    Html(generate_redact_page("", ""))
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
                            <button class="bluebutton" type="button" onclick="window.location.href='/legacy/'">← Back to list</button>
        </div>
        
        {message_html}
        
        <div class="section">
            <p>Enter passwords to redact from all log entries. Each password should be on a separate line.</p>
            <p><strong>Warning:</strong> This operation will permanently modify your log data. Make sure you have a backup.</p>
            
            <div class="form-group">
                <label for="passwords">Passwords to redact (one per line):</label>
                <textarea id="passwords" name="passwords" rows="10" cols="50" placeholder="password123&#10;secret456&#10;mytoken789">{passwords}</textarea>
            </div>
            <div class="form-group">
                <button class="redbutton" type="button" onclick="performRedaction()">Redact Passwords</button>
                <button class="greybutton" type="button" onclick="document.getElementById('passwords').value = ''">Clear</button>
            </div>
        </div>
    </div>
    <script>
        async function performRedaction() {{
            const passwordsText = document.getElementById('passwords').value.trim();
            if (!passwordsText) {{
                alert('Please enter at least one password to redact.');
                return;
            }}
            
            const passwords = passwordsText.split('\\n').map(line => line.trim()).filter(line => line);
            if (passwords.length === 0) {{
                alert('Please enter at least one valid password to redact.');
                return;
            }}
            
            const message = `Are you sure you want to redact ${{passwords.length}} password(s) from all log entries? This operation will permanently modify your log data and cannot be undone.`;
            if (!confirm(message)) {{
                return;
            }}
            
            // Disable the button and show loading state
            const button = event.target;
            const originalText = button.textContent;
            button.disabled = true;
            button.textContent = 'Redacting...';
            
            try {{
                const response = await fetch('/redact', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify({{ passwords: passwords }})
                }});
                
                const result = await response.json();
                
                if (response.ok) {{
                    showMessage(result.message, 'success');
                    document.getElementById('passwords').value = '';
                }} else {{
                    showMessage(result.error || 'Failed to redact passwords', 'error');
                }}
            }} catch (error) {{
                showMessage('Error performing redaction: ' + error.message, 'error');
            }} finally {{
                button.disabled = false;
                button.textContent = originalText;
            }}
        }}
        
        function showMessage(message, type) {{
            // Remove any existing message
            const existingMessage = document.querySelector('.message');
            if (existingMessage) {{
                existingMessage.remove();
            }}
            
            // Create new message div
            const messageDiv = document.createElement('div');
            messageDiv.className = `message ${{type}}`;
            messageDiv.textContent = message;
            
            // Insert after the header
            const header = document.querySelector('.header');
            header.parentNode.insertBefore(messageDiv, header.nextSibling);
            
            // Auto-hide success messages after 5 seconds
            if (type === 'success') {{
                setTimeout(() => {{
                    if (messageDiv.parentNode) {{
                        messageDiv.remove();
                    }}
                }}, 5000);
            }}
        }}
    </script>
</body>
</html>
"#)
} 