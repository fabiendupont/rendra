use std::path::Path;
use std::process::Command;

use runtime_sandbox::manifest::{AppManifest, ManifestError};

/// Errors that can occur during dev build and run.
#[derive(Debug, thiserror::Error)]
pub enum DevError {
    #[error("no app.toml found in {0}")]
    NoManifest(std::path::PathBuf),

    #[error("failed to parse manifest: {0}")]
    Manifest(#[from] ManifestError),

    #[error("build failed")]
    Build,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Build and run the application in the given project directory.
pub fn run_dev(project_dir: &Path) -> Result<(), DevError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(DevError::NoManifest(project_dir.to_path_buf()));
    }

    let manifest = AppManifest::from_file(&manifest_path)?;
    let app_name = &manifest.app.name;

    println!("Building {app_name}...");

    let status = Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .status()?;

    if !status.success() {
        return Err(DevError::Build);
    }

    let bin_path = project_dir.join(format!("target/debug/{app_name}"));

    println!("Launching {}...", bin_path.display());

    let status = Command::new(&bin_path)
        .current_dir(project_dir)
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
