use axum::{extract::{Path, State}, response::Html};
use crate::{model::CaptureType, ui::{rest::get_entry, ProdlogUiState}};

const OUTPUT_CONTENT: &str = r#"
    <link rel="stylesheet" href="/static/xterm.css" />
    <script src="/static/xterm.js"></script>
    <script>
        // Fetch the entry data first
        window.prodlog.get_prodlog_entry()
            .then(entry => {
                // Create terminal with the correct dimensions
                const term = new Terminal({
                    cols: entry.terminal_cols || 120,
                    rows: entry.terminal_rows || 40,
                    cursorBlink: true,
                    scrollback: 9999999,
                    fontSize: 14,
                    fontFamily: 'monospace',
                    convertEol: true,
                    disableStdin: true
                });
                // Don't let xterm.js handle any key events
                term.attachCustomKeyEventHandler((event) => false);
                term.open(document.getElementById('terminal'));

                // Decode base64 to raw bytes
                const binaryString = atob(entry.captured_output);
                const bytes = new Uint8Array(binaryString.length);
                for (let i = 0; i < binaryString.length; i++) {
                    bytes[i] = binaryString.charCodeAt(i);
                }
                
                // Write raw bytes to terminal
                term.write(bytes);
            })
            .catch(error => {
                document.body.innerHTML = `<div style="color: red; padding: 20px;">Error: ${error.message}</div>`;
            });
    </script>
    <div class="section">
        <h2 id="header-title">Output</h2>
        <div id="terminal"/>
    </div>
    </div>
    "#;

const DIFF_CONTENT: &str = r#"
    <div class="section">
        <h2 id="header-title">File Diff</h2>
        <pre class="diff-output" id="diff-content"></pre>
    </div>
    <script>
        // Fetch the entry data first
        window.prodlog.get_prodlog_diffcontent()
            .then(diff => {
                document.getElementById('diff-content').innerHTML = diff.diff;
            })
            .catch(error => {
                document.body.innerHTML = `<div style="color: red; padding: 20px;">Error: ${error.message}</div>`;
            });
    </script>"#;

const EDIT_CONTENT: &str = r#"
    <div class="edit-sections-container" id="edit-sections" style="display: none;">
        <div class="section comment-section">
            <h2>Comment</h2>
            <form id="editForm">
                <div>
                    <textarea name="message" id="edit-message" rows="5">Loading...</textarea>
                </div>
                <div class="button-group">
                    <button type="submit" class="bluebutton">Update Comment</button>
                </div>
            </form>
        </div>
        <div class="section noop-section">
            <h2>No-op</h2>
            <div class="noop-toggle-container">
                <div class="switch-container">
                    <label class="switch">
                        <input type="checkbox" id="noop-switch" onchange="toggleNoop(this)">
                        <span class="slider"></span>
                    </label>
                    <span class="switch-label" id="noop-status">Loading...</span>
                </div>
            </div>
        </div>
        <div class="section redact-section">
            <h2>Redact Password</h2>
            <p>Remove a password from this specific entry. This will replace all occurrences of the password with [REDACTED].</p>
            <form id="redactForm">
                <div>
                    <label for="password">Password to redact:</label>
                    <input  type="text" name="password" id="redact-password" placeholder="Enter password to redact">
                </div>
                <div class="button-group">
                    <button type="submit" class="redbutton">Redact Password</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        let currentEntry = null;
        
        // Add toggle button to header on page load
        document.addEventListener('DOMContentLoaded', function() {
            const headerDiv = document.querySelector('.container > div');
            headerDiv.style.display = 'flex';
            headerDiv.style.justifyContent = 'space-between';
            headerDiv.style.alignItems = 'center';
            headerDiv.style.marginBottom = '1.5rem';
            
            const toggleButton = document.createElement('button');
            toggleButton.id = 'edit-toggle-btn';
            toggleButton.className = 'bluebutton';
            toggleButton.onclick = toggleEditSections;
            toggleButton.innerHTML = '<span id="edit-toggle-icon">▼</span><span id="edit-toggle-text">Edit</span>';
            toggleButton.style.display = 'inline-flex';
            toggleButton.style.alignItems = 'center';
            toggleButton.style.gap = '0.5rem';
            
            headerDiv.appendChild(toggleButton);
        });
        
        window.prodlog.get_prodlog_entry()
                .then(entry => {
                    currentEntry = entry;
                    document.getElementById("edit-message").textContent = entry.message;
                    updateNoopStatus(entry.is_noop);
                    
                    // Handle comment form submission
                    document.getElementById('editForm').addEventListener('submit', async (e) => {
                        e.preventDefault();
                        const form = e.target;
                        const data = {
                            uuid: entry.uuid,
                            message: form.message.value,
                            is_noop: entry.is_noop // Keep current noop status
                        };
                        try {
                            const response = await fetch('/api/entry', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                },
                                body: JSON.stringify(data)
                            });
                            if (response.ok) {
                                // Update entry object with new message
                                currentEntry.message = form.message.value;
                                showMessage('Comment updated successfully', 'success');
                            } else {
                                showMessage('Failed to update comment', 'error');
                            }
                        } catch (error) {
                            showMessage('Error updating comment: ' + error, 'error');
                        }
                    });

                    // Handle redact form
                    document.getElementById('redactForm').addEventListener('submit', async (e) => {
                        e.preventDefault();
                        const form = e.target;
                        const password = form.password.value.trim();
                        
                        if (!password) {
                            showMessage('Please enter a password to redact.', 'error');
                            return;
                        }

                        if (!confirm(`Are you sure you want to redact the password "${password}" from this entry? This operation will permanently modify the entry data and cannot be undone.`)) {
                            return;
                        }

                        try {
                            const response = await fetch('/api/entry/redact', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                },
                                body: JSON.stringify({
                                    uuid: entry.uuid,
                                    password: password
                                })
                            });
                            
                            const result = await response.json();
                            
                            if (response.ok) {
                                showMessage(result.message, 'success');
                                form.password.value = '';
                            } else {
                                showMessage(result.error || 'Failed to redact password', 'error');
                            }
                        } catch (error) {
                            showMessage('Error redacting password: ' + error, 'error');
                        }
                    });
                });

        async function toggleNoop(switchElement) {
            if (!currentEntry) return;
            
            const newNoopStatus = switchElement.checked;
            
            // Disable switch during update
            const statusSpan = document.getElementById('noop-status');
            const originalText = statusSpan.textContent;
            switchElement.disabled = true;
            statusSpan.textContent = 'Updating...';
            
                            try {
                    const response = await fetch('/api/entry', {
                        method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        uuid: currentEntry.uuid,
                        message: currentEntry.message,
                        is_noop: newNoopStatus
                    })
                });
                
                const is_noop = newNoopStatus ? 'NO-OP' : 'NOT a no-op';
                if (response.ok) {
                    currentEntry.is_noop = newNoopStatus;
                    updateNoopStatus(newNoopStatus);
                    showMessage(`Entry successfully marker as ${is_noop}.`, 'success');
                } else {
                    // Revert switch state on error
                    switchElement.checked = !newNoopStatus;
                    showMessage(`Failed mark entry as ${is_noop}.`, 'error');
                }
            } catch (error) {
                // Revert switch state on error
                switchElement.checked = !newNoopStatus;
                showMessage(`Error updating entry: ${error}`, 'error');
            } finally {
                switchElement.disabled = false;
                if (statusSpan.textContent === 'Updating...') {
                    statusSpan.textContent = originalText;
                }
            }
        }
        
        function updateNoopStatus(isNoop) {
            const statusSpan = document.getElementById('noop-status');
            const switchElement = document.getElementById('noop-switch');
            
            switchElement.checked = isNoop;
            statusSpan.textContent = isNoop ? 'Marked as no-op. This entry had no effect.' : 'Not a no-op.';
        }
        
        function showMessage(message, type) {
            // Remove any existing message
            const existingMessage = document.querySelector('.temp-message');
            if (existingMessage) {
                existingMessage.remove();
            }
            
            // Create new message div
            const messageDiv = document.createElement('div');
            messageDiv.className = `message ${type} temp-message`;
            messageDiv.textContent = message;
            
            // Insert at the top of the container
            const container = document.querySelector('.container');
            const firstSection = container.querySelector('.section');
            container.insertBefore(messageDiv, firstSection);
        }

        function toggleEditSections() {
            const editSections = document.getElementById('edit-sections');
            const toggleIcon = document.getElementById('edit-toggle-icon');
            const toggleText = document.getElementById('edit-toggle-text');
            
            if (editSections.style.display === 'none') {
                editSections.style.display = 'flex';
                toggleIcon.textContent = '▲';
                toggleText.textContent = 'Edit';
            } else {
                editSections.style.display = 'none';
                toggleIcon.textContent = '▼';
                toggleText.textContent = 'Edit';
            }
        }


    </script>
    "#;

fn generate_detail_page(title: &str, content: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="/prodlog-dyn.css">
    <link rel="stylesheet" href="/static/prodlog.css">
    <script src="/static/prodlog.js"></script>
</head>
<body>
    <div class="container">
        <div>
                            <button class="bluebutton" type="button" onclick="window.location.href='/legacy/'">← Back to list</button>
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

pub async fn handle_entry(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> Html<String> {
    let entry = match get_entry(sink.clone(), &uuid).await {
        Ok(entry) => entry,
        Err((_, message)) => return Html(message),
    };

    let content = match entry.capture_type {
        CaptureType::Run => {
            EDIT_CONTENT.to_owned() + OUTPUT_CONTENT
        }
        CaptureType::Edit => {
            EDIT_CONTENT.to_owned() + DIFF_CONTENT
        }
    };

    Html(generate_detail_page("Entry", &content))
}
