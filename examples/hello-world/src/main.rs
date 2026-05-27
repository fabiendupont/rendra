use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");

    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("Hello World")
        .size(800, 600)
        .url(url)
        .run()
}
