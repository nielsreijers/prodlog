use axum::{extract::{Path, State}, response::Html};
use crate::{model::CaptureType, ui::{pages::entry::common::generate_detail_page, rest::get_entry, ProdlogUiState}};

pub const EDIT_CONTENT: &str = r#"
    <div class="edit-sections-container">
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
            <div id="redact-message" class="message" style="display: none;"></div>
        </div>
    </div>
    <script>
        let currentEntry = null;
        
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
                            const response = await fetch('/entry', {
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
                            showRedactMessage('Please enter a password to redact.', 'error');
                            return;
                        }

                        if (!confirm(`Are you sure you want to redact the password "${password}" from this entry? This operation will permanently modify the entry data and cannot be undone.`)) {
                            return;
                        }

                        try {
                            const response = await fetch('/entry/redact', {
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
                                showRedactMessage(result.message, 'success');
                                form.password.value = '';
                                // Refresh the page to show updated content
                                setTimeout(() => {
                                    window.location.reload();
                                }, 1500);
                            } else {
                                showRedactMessage(result.error || 'Failed to redact password', 'error');
                            }
                        } catch (error) {
                            showRedactMessage('Error redacting password: ' + error, 'error');
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
                const response = await fetch('/entry', {
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
            
            // Auto-hide after 3 seconds
            setTimeout(() => {
                if (messageDiv.parentNode) {
                    messageDiv.remove();
                }
            }, 3000);
        }

        function showRedactMessage(message, type) {
            const messageDiv = document.getElementById('redact-message');
            messageDiv.textContent = message;
            messageDiv.className = `message ${type}`;
            messageDiv.style.display = 'block';
            
            // Hide message after 5 seconds
            setTimeout(() => {
                messageDiv.style.display = 'none';
            }, 5000);
        }
    </script>
    "#;

pub async fn handle_edit(
    State(sink): State<ProdlogUiState>,
    Path(uuid): Path<String>,
) -> Html<String> {
    let entry = match get_entry(sink.clone(), &uuid).await {
        Ok(entry) => entry,
        Err((_, message)) => return Html(message),
    };

    let content = match entry.capture_type {
        CaptureType::Run => {
            EDIT_CONTENT.to_owned() + crate::ui::pages::entry::output::OUTPUT_CONTENT
        }
        CaptureType::Edit => {
            EDIT_CONTENT.to_owned() + crate::ui::pages::entry::diff::DIFF_CONTENT
        }
    };

    Html(generate_detail_page("Edit Entry", &content))
}
