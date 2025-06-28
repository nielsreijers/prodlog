use axum::{extract::{Path, State}, response::Html};
use crate::{model::CaptureType, ui::{pages::entry::common::generate_detail_page, rest::get_entry, ProdlogUiState}};

pub const EDIT_CONTENT: &str = r#"
    <div class="section">
        <h2 id="header-title">Edit entry</h2>
        <form id="editForm">
            <div>
                <label for="message">Message:</label>
                <textarea name="message" id="edit-message" rows="5">Loading...</textarea>
            </div>
            <div class="switch-container">
                <label class="switch">
                    <input type="checkbox" name="is_noop" id="edit-is-noop">
                    <span class="slider"></span>
                </label>
                <span class="switch-label">Mark as no-op (this command had no effect)</span>
            </div>
            <div class="button-group">
                <button type="submit" class="bluebutton">Save</button>
                <a href="/" class="greybutton">Cancel</a>
            </div>
        </form>
    </div>
    <div class="section">
        <h2>Redact Password</h2>
        <p>Remove a password from this specific entry. This will replace all occurrences of the password with [REDACTED].</p>
        <form id="redactForm">
            <div>
                <label for="password">Password to redact:</label>
                <input type="text" name="password" id="redact-password" placeholder="Enter password to redact">
            </div>
            <div class="button-group">
                <button type="submit" class="redbutton">Redact Password</button>
            </div>
        </form>
        <div id="redact-message" class="message" style="display: none;"></div>
    </div>
    <script>
        window.prodlog.get_prodlog_entry()
                .then(entry => {
                    document.getElementById("edit-message").textContent = entry.message;
                    document.getElementById("edit-is-noop").checked = entry.is_noop;
                    document.getElementById('editForm').addEventListener('submit', async (e) => {
                        e.preventDefault();
                        const form = e.target;
                        const data = {
                            uuid: entry.uuid,
                            message: form.message.value,
                            is_noop: form.is_noop.checked
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
                                window.location.href = '/';
                            } else {
                                alert('Failed to save changes');
                            }
                        } catch (error) {
                            alert('Error saving changes: ' + error);
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
