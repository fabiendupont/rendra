use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use runtime_sandbox::manifest::AppManifest;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("manifest error: {0}")]
    Manifest(#[from] runtime_sandbox::manifest::ManifestError),
    #[error("no app.toml found")]
    NoManifest,
    #[error("build failed: {0}")]
    Build(String),
    #[error("binary not found at {0}")]
    BinaryNotFound(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn run_package(project_dir: &Path) -> Result<PathBuf, PackageError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(PackageError::NoManifest);
    }
    let manifest = AppManifest::from_file(&manifest_path)?;
    let app_name = &manifest.app.name;
    let app_version = &manifest.app.version;

    println!("Building {app_name} (release)...");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(PackageError::Build("cargo build --release failed".into()));
    }

    let bin_path = project_dir.join("target").join("release").join(app_name);
    if !bin_path.exists() {
        return Err(PackageError::BinaryNotFound(bin_path));
    }

    let appdir = project_dir.join("target").join("appimage");
    let appdir_root = appdir.join(format!("{app_name}.AppDir"));
    let usr_bin = appdir_root.join("usr").join("bin");
    let usr_share = appdir_root.join("usr").join("share").join("applications");
    let usr_icons = appdir_root.join("usr").join("share").join("icons").join("hicolor").join("256x256").join("apps");

    std::fs::create_dir_all(&usr_bin)?;
    std::fs::create_dir_all(&usr_share)?;
    std::fs::create_dir_all(&usr_icons)?;

    std::fs::copy(&bin_path, usr_bin.join(app_name))?;

    let desktop_entry = format!(
        "[Desktop Entry]\nType=Application\nName={app_name}\nExec={app_name}\nIcon={app_name}\nCategories=Utility;\n"
    );
    std::fs::write(usr_share.join(format!("{app_name}.desktop")), &desktop_entry)?;
    std::fs::write(appdir_root.join(format!("{app_name}.desktop")), &desktop_entry)?;

    let icon_src = project_dir.join("assets").join("icon.png");
    if icon_src.exists() {
        std::fs::copy(&icon_src, usr_icons.join(format!("{app_name}.png")))?;
        std::fs::copy(&icon_src, appdir_root.join(format!("{app_name}.png")))?;
    } else {
        create_placeholder_icon(&usr_icons.join(format!("{app_name}.png")))?;
        create_placeholder_icon(&appdir_root.join(format!("{app_name}.png")))?;
    }

    let apprun = format!("#!/bin/sh\nexec \"$APPDIR/usr/bin/{app_name}\" \"$@\"\n");
    let apprun_path = appdir_root.join("AppRun");
    std::fs::write(&apprun_path, apprun)?;
    std::fs::set_permissions(&apprun_path, std::fs::Permissions::from_mode(0o755))?;

    println!("AppDir created at {}", appdir_root.display());

    if let Ok(tool) = which_appimagetool() {
        let output_name = format!("{app_name}-{app_version}-x86_64.AppImage");
        let output_path = appdir.join(&output_name);
        println!("Running appimagetool...");
        let status = Command::new(&tool)
            .arg(&appdir_root)
            .arg(&output_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if status.success() {
            let size_mb = std::fs::metadata(&output_path)?.len() as f64 / 1_048_576.0;
            println!("AppImage created: {} ({:.1} MB)", output_path.display(), size_mb);
            return Ok(output_path);
        }
        eprintln!("appimagetool failed, but AppDir is ready at {}", appdir_root.display());
    } else {
        println!("appimagetool not found. To create an AppImage, install it and run:");
        println!("  appimagetool {} {app_name}-{app_version}-x86_64.AppImage", appdir_root.display());
    }

    Ok(appdir_root)
}

fn which_appimagetool() -> Result<PathBuf, ()> {
    Command::new("which")
        .arg("appimagetool")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let path = String::from_utf8(o.stdout).ok()?.trim().to_string();
            Some(PathBuf::from(path))
        })
        .ok_or(())
}

fn create_placeholder_icon(path: &Path) -> std::io::Result<()> {
    // 1x1 transparent PNG
    let png: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
        0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02,
        0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::write(path, png)
}
