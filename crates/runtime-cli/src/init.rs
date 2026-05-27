use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur during project scaffolding.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("directory already exists and is not empty: {0}")]
    DirectoryExists(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Scaffold a new project at the given path with the given name.
///
/// Creates the directory structure:
/// - `app.toml` — application manifest
/// - `Cargo.toml` — Rust package manifest
/// - `src/main.rs` — application entry point
/// - `frontend/index.html` — minimal welcome page
/// - `assets/` — static assets directory
pub fn scaffold_project(path: &Path, name: &str) -> Result<(), InitError> {
    if path.exists() {
        let has_entries = path.read_dir()?.next().is_some();
        if has_entries {
            return Err(InitError::DirectoryExists(path.to_path_buf()));
        }
    } else {
        fs::create_dir_all(path)?;
    }

    // app.toml
    let app_toml = format!(
        r#"[app]
name = "{name}"
version = "0.1.0"

[window]
title = "{name}"
width = 1024
height = 768
"#
    );
    fs::write(path.join("app.toml"), app_toml)?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
runtime-core = {{ git = "https://github.com/nicoburniske/servo-runtime.git" }}
url = "2"
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    // src/main.rs
    let main_rs = r#"use runtime_core::AppBuilder;
use url::Url;

fn main() {
    let html = Url::parse("file://frontend/index.html").expect("valid URL");
    AppBuilder::new()
        .with_url(html)
        .run();
}
"#;
    fs::create_dir_all(path.join("src"))?;
    fs::write(path.join("src/main.rs"), main_rs)?;

    // frontend/index.html
    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name}</title>
</head>
<body>
    <h1>Welcome to {name}</h1>
    <p>Edit <code>frontend/index.html</code> to get started.</p>
</body>
</html>
"#
    );
    fs::create_dir_all(path.join("frontend"))?;
    fs::write(path.join("frontend/index.html"), index_html)?;

    // assets/
    fs::create_dir_all(path.join("assets"))?;

    Ok(())
}
