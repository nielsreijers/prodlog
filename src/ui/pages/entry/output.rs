use axum::response::Html;
use crate::ui::pages::entry::common::generate_detail_page;

pub const OUTPUT_CONTENT: &str = r#"
    <link rel="stylesheet" href="/static/xterm.css" />
    <script src="/static/xterm.js"></script>
    <script>
        // Fetch the entry data first
        window.prodlog.get_prodlog_entry()
            .then(entry => {
                // Create terminal with the correct dimensions
                const term = new Terminal({
                    cols: entry.terminal_cols,
                    rows: entry.terminal_rows,
                    cursorBlink: true,
                    scrollback: 1000,
                    fontSize: 14,
                    fontFamily: 'monospace',
                    convertEol: true,
                    disableStdin: true
                });
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

pub async fn handle_output(
) -> Html<String> {
    Html(generate_detail_page("Output View", OUTPUT_CONTENT))
}
