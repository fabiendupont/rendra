use std::path::PathBuf;

use runtime_ipc::command::CommandHandler;
use runtime_ipc::IpcError;
use serde_json::{json, Value};

fn notes_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("markdown-notes");
    std::fs::create_dir_all(&dir).ok();
    dir
}

struct RenderMarkdownHandler;

impl CommandHandler for RenderMarkdownHandler {
    fn handle(&self, args: Value) -> Result<Value, IpcError> {
        let markdown = args
            .get("markdown")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        let parser = pulldown_cmark::Parser::new_ext(markdown, options);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        Ok(json!({ "html": html }))
    }
}

struct ListNotesHandler {
    dir: PathBuf,
}

impl ListNotesHandler {
    fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl CommandHandler for ListNotesHandler {
    fn handle(&self, _args: Value) -> Result<Value, IpcError> {
        let mut notes: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(stem) = path.file_stem() {
                        notes.push(stem.to_string_lossy().into_owned());
                    }
                }
            }
        }
        notes.sort();
        Ok(json!({ "notes": notes }))
    }
}

struct SaveNoteHandler {
    dir: PathBuf,
}

impl SaveNoteHandler {
    fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl CommandHandler for SaveNoteHandler {
    fn handle(&self, args: Value) -> Result<Value, IpcError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IpcError::InvalidArgs("missing 'name'".into()))?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(IpcError::InvalidArgs("invalid note name".into()));
        }

        let path = self.dir.join(format!("{name}.md"));
        std::fs::write(&path, content)
            .map_err(|e| IpcError::HandlerError(e.to_string()))?;
        Ok(json!({ "saved": true }))
    }
}

struct LoadNoteHandler {
    dir: PathBuf,
}

impl LoadNoteHandler {
    fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl CommandHandler for LoadNoteHandler {
    fn handle(&self, args: Value) -> Result<Value, IpcError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IpcError::InvalidArgs("missing 'name'".into()))?;

        let path = self.dir.join(format!("{name}.md"));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| IpcError::HandlerError(e.to_string()))?;
        Ok(json!({ "name": name, "content": content }))
    }
}

struct DeleteNoteHandler {
    dir: PathBuf,
}

impl DeleteNoteHandler {
    fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl CommandHandler for DeleteNoteHandler {
    fn handle(&self, args: Value) -> Result<Value, IpcError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IpcError::InvalidArgs("missing 'name'".into()))?;

        let path = self.dir.join(format!("{name}.md"));
        std::fs::remove_file(&path)
            .map_err(|e| IpcError::HandlerError(e.to_string()))?;
        Ok(json!({ "deleted": true }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = notes_dir();

    let html_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");
    let url = url::Url::from_file_path(&html_path)
        .map_err(|_| format!("Invalid path: {}", html_path.display()))?;

    runtime_core::AppBuilder::new()
        .title("Markdown Notes")
        .size(1200, 800)
        .url(url)
        .command("render_markdown", RenderMarkdownHandler)
        .command("list_notes", ListNotesHandler::new(dir.clone()))
        .command("save_note", SaveNoteHandler::new(dir.clone()))
        .command("load_note", LoadNoteHandler::new(dir.clone()))
        .command("delete_note", DeleteNoteHandler::new(dir))
        .run()
}
