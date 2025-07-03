use std::path::Path;

fn main() {
    // Rerun if React source files change
    println!("cargo:rerun-if-changed=react-ui/src");
    println!("cargo:rerun-if-changed=react-ui/package.json");
    println!("cargo:rerun-if-changed=react-ui/package-lock.json");
    
    // Check if React build output exists and warn if not
    if !Path::new("src/ui/static/react/assets").exists() 
        || !Path::new("src/ui/static/react/index.html").exists() 
        || std::fs::read_to_string("src/ui/static/react/index.html")
            .unwrap_or_default()
            .contains("React UI is being built") {
        println!("cargo:warning=React UI not built. Run './build-react.sh' to build the React frontend.");
    }
} 