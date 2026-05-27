use std::fs;

use runtime_cli::init::{InitError, scaffold_project};

#[test]
fn init_creates_project_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("my-app");

    scaffold_project(&project_dir, "my-app").unwrap();

    // Verify all expected files exist
    assert!(project_dir.join("app.toml").is_file());
    assert!(project_dir.join("Cargo.toml").is_file());
    assert!(project_dir.join("src/main.rs").is_file());
    assert!(project_dir.join("frontend/index.html").is_file());
    assert!(project_dir.join("assets").is_dir());

    // Verify content contains the project name
    let app_toml = fs::read_to_string(project_dir.join("app.toml")).unwrap();
    assert!(app_toml.contains("name = \"my-app\""));

    let cargo_toml = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"my-app\""));

    let main_rs = fs::read_to_string(project_dir.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("AppBuilder"));

    let index_html = fs::read_to_string(project_dir.join("frontend/index.html")).unwrap();
    assert!(index_html.contains("Welcome to my-app"));

    // Verify window title in app.toml
    assert!(app_toml.contains("title = \"my-app\""));
}

#[test]
fn init_fails_if_directory_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("existing-app");

    // Create a non-empty directory
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("some-file.txt"), "content").unwrap();

    let result = scaffold_project(&project_dir, "existing-app");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, InitError::DirectoryExists(_)));
}
