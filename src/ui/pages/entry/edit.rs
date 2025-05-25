use axum::response::Html;
use crate::ui::pages::entry::common::generate_detail_page;

pub async fn handle_edit() -> Html<String> {
    let content = format!(r#"
        <form id="editForm">
            <div class="form-group">
                <label for="message">Message:</label>
                <textarea name="message" id="edit-message" rows="10">Loading...</textarea>
            </div>
            <div class="switch-container">
                <label class="switch">
                    <input type="checkbox" name="is_noop" id="edit-is-noop">
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
            window.prodlog.get_prodlog_entry()
                .then(entry => {{
                    document.getElementById("edit-message").textContent = entry.message;
                    document.getElementById("edit-is-noop").checked = entry.is_noop;
                    document.getElementById('editForm').addEventListener('submit', async (e) => {{
                        e.preventDefault();
                        const form = e.target;
                        const data = {{
                            uuid: entry.uuid,
                            message: form.message.value,
                            is_noop: form.is_noop.checked
                        }};
                        try {{
                            const response = await fetch('/entry', {{
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
                }});
        </script>
    "#);
    Html(generate_detail_page("Edit Entry", &content))
}
