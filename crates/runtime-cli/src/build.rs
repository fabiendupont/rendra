use std::path::Path;
use std::process::Command;

use runtime_sandbox::manifest::{AppManifest, ManifestError};

/// Errors that can occur during a release build.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("no app.toml found in {0}")]
    NoManifest(std::path::PathBuf),

    #[error("failed to parse manifest: {0}")]
    Manifest(#[from] ManifestError),

    #[error("build failed")]
    Build,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Build the application in release mode.
pub fn run_build(project_dir: &Path) -> Result<(), BuildError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(BuildError::NoManifest(project_dir.to_path_buf()));
    }

    let manifest = AppManifest::from_file(&manifest_path)?;
    let app_name = &manifest.app.name;

    println!("Building {app_name} (release)...");

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()?;

    if !status.success() {
        return Err(BuildError::Build);
    }

    let bin_path = project_dir.join(format!("target/release/{app_name}"));

    if bin_path.exists() {
        let metadata = std::fs::metadata(&bin_path)?;
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        println!("Binary: {} ({:.2} MB)", bin_path.display(), size_mb);
    } else {
        println!("Binary: {}", bin_path.display());
    }

    Ok(())
}
