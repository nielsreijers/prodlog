pub fn generate_detail_page(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="/static/prodlog.css">
    <script src="/static/prodlog.js"></script>
</head>
<body>
    <div class="container">
        <div>
            <button class="bluebutton" type="button" onclick="window.location.href='/'">← Back to list</button>
        </div>
        <script>
        window.prodlog.get_prodlog_entry()
            .then(entry => {{
                const updates = {{
                    'header-host': entry.host,
                    'header-cmd': entry.cmd,
                    'header-cwd': entry.cwd,
                    'header-start': new Date(entry.start_time).toLocaleString(),
                    'header-end': new Date(new Date(entry.start_time).getTime() + entry.duration_ms).toLocaleString(),
                    'header-duration': entry.duration_ms,
                    'header-exit': entry.exit_code
                }};

                for (const [key, value] of Object.entries(updates)) {{
                    document.getElementById(key).textContent = value;
                }}

                // Handle message
                if (entry.message && entry.message.trim()) {{
                    const messageDiv = document.createElement('div');
                    messageDiv.className = 'message';
                    messageDiv.textContent = entry.message;
                    document.getElementById('header-info').appendChild(messageDiv);
                }}

                if (entry.capture_type == "Run") {{
                    document.getElementById('header-title').textContent = entry.cmd;
                }} else {{
                    document.getElementById('header-title').textContent = entry.filename;
                }}
            }});
        </script>
        <div class="section" id="header-info">
            <h2 id="header-title">Loading...</h2>
            <div class="info-grid">
                <div class="info-item">
                    <span class="info-label">Host:</span>
                    <span class="info-value" id="header-host">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Command:</span>
                    <span class="info-value" id="header-cmd">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Directory:</span>
                    <span class="info-value" id="header-cwd">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Start:</span>
                    <span class="info-value" id="header-start">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">End:</span>
                    <span class="info-value" id="header-end">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Duration:</span>
                    <span class="info-value" id="header-duration">Loading...</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Exit Code:</span>
                    <span class="info-value" id="header-exit">Loading...</span>
                </div>
            </div>
        </div>
        {content}
    </div>
</body>
</html>
    "#)
}
