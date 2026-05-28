use std::path::PathBuf;

use runtime_ipc::command::CommandHandler;
use serde_json::{json, Value};

struct ToggleThemeHandler;

impl CommandHandler for ToggleThemeHandler {
    fn handle(&self, _args: Value) -> Result<Value, runtime_ipc::IpcError> {
        Ok(json!({ "toggled": true }))
    }
}

struct ShowToastHandler;

impl CommandHandler for ShowToastHandler {
    fn handle(&self, args: Value) -> Result<Value, runtime_ipc::IpcError> {
        let msg = args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello!");
        Ok(json!({ "message": msg }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");

    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("Rendra UI Showcase")
        .size(1400, 900)
        .url(url)
        .command("toggle_theme", ToggleThemeHandler)
        .command("show_toast", ShowToastHandler)
        .run()
}
