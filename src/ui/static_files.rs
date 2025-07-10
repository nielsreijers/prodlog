use include_dir::{include_dir, Dir};
use std::path::Path;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    extract::Path as AxumPath,
};

// Include the static directory at compile time
static STATIC_DIR: Dir<'_> = include_dir!("src/ui/static");

// Include the React build directory at compile time  
static REACT_DIR: Dir<'_> = include_dir!("src/ui/static/react");

pub async fn serve_file(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    // Try to get the file from the embedded directory
    if let Some(file) = STATIC_DIR.get_file(&path) {
        // Determine content type based on file extension
        let content_type = match Path::new(&path).extension().and_then(|ext| ext.to_str()) {
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            _ => "application/octet-stream",
        };

        // Return the file with appropriate headers
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", content_type)
            .body(Body::from(file.contents()))
            .unwrap()
    } else {
        // File not found
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not found"))
            .unwrap()
    }
}

pub async fn serve_react_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    // Try to get the file from the embedded React directory
    let asset_path = format!("assets/{}", path);
    
    if let Some(file) = REACT_DIR.get_file(&asset_path) {
            // Determine content type based on file extension
            let content_type = match Path::new(&path).extension().and_then(|ext| ext.to_str()) {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                Some("woff") => "font/woff",
                Some("woff2") => "font/woff2",
                _ => "application/octet-stream",
            };

            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
            .body(Body::from(file.contents()))
                .unwrap()
    } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Asset not found"))
                .unwrap()
    }
}

pub async fn serve_react_favicon() -> impl IntoResponse {
    // Try to get favicon from the embedded React directory
    if let Some(file) = REACT_DIR.get_file("favicon.ico") {
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "image/x-icon")
            .body(Body::from(file.contents()))
                .unwrap()
    } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Favicon not found"))
                .unwrap()
        }
    }

// Export the REACT_DIR so it can be used in mod.rs
pub fn get_react_index_html() -> Option<String> {
    REACT_DIR.get_file("index.html")
        .map(|file| std::str::from_utf8(file.contents()).unwrap().to_string())
}
 