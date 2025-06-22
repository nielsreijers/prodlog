use axum::response::Html;
use crate::ui::pages::entry::common::generate_detail_page;

pub const DIFF_CONTENT: &str = r#"
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

pub async fn handle_diff(
) -> Html<String> {
    Html(generate_detail_page("File Diff", &DIFF_CONTENT))
}
