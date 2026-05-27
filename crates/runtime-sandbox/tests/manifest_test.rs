use runtime_sandbox::manifest::AppManifest;

#[test]
fn parse_minimal_manifest() {
    let toml_str = r#"
[app]
name = "test-app"
version = "0.1.0"
"#;

    let manifest: AppManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.app.name, "test-app");
    assert_eq!(manifest.app.version, "0.1.0");
    assert!(manifest.app.description.is_none());
    assert!(manifest.window.is_none());
    assert!(manifest.permissions.is_none());
    assert!(manifest.build.is_none());
}

#[test]
fn parse_full_manifest() {
    let toml_str = r#"
[app]
name = "my-app"
version = "1.0.0"
description = "A test application"

[window]
title = "My App"
width = 1280
height = 720
resizable = true
decorations = false

[permissions]
network = ["https://api.example.com"]
clipboard = ["read", "write"]

[permissions.filesystem]
user-files = "portal"

[build]
frontend = "npm run build"
assets = ["dist", "static"]
"#;

    let manifest: AppManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.app.name, "my-app");
    assert_eq!(manifest.app.version, "1.0.0");
    assert_eq!(manifest.app.description.as_deref(), Some("A test application"));

    let window = manifest.window.as_ref().unwrap();
    assert_eq!(window.title.as_deref(), Some("My App"));
    assert_eq!(window.width, Some(1280));
    assert_eq!(window.height, Some(720));
    assert_eq!(window.resizable, Some(true));
    assert_eq!(window.decorations, Some(false));

    let perms = manifest.permissions.as_ref().unwrap();
    assert_eq!(perms.network.as_ref().unwrap(), &["https://api.example.com"]);
    assert_eq!(perms.clipboard.as_ref().unwrap(), &["read", "write"]);

    let fs = perms.filesystem.as_ref().unwrap();
    assert!(fs.user_files.is_some());

    let build = manifest.build.as_ref().unwrap();
    assert_eq!(build.frontend.as_deref(), Some("npm run build"));
    assert_eq!(build.assets.as_ref().unwrap(), &["dist", "static"]);
}

#[test]
fn reject_unknown_permission() {
    let toml_str = r#"
[app]
name = "test-app"
version = "0.1.0"

[permissions]
webcam = true
"#;

    let result: Result<AppManifest, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}
