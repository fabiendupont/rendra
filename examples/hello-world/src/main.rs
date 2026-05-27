use std::path::PathBuf;

use runtime_ipc::command::CommandHandler;
use serde_json::{json, Value};

struct GreetHandler;

impl CommandHandler for GreetHandler {
    fn handle(&self, args: Value) -> Result<Value, runtime_ipc::IpcError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(json!({ "greeting": format!("Hello, {}!", name) }))
    }
}

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
        .command("greet", GreetHandler)
        .run()
}
