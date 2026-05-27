use std::path::Path;

use runtime_api::filesystem::ScopedFs;
use tempfile::TempDir;

#[tokio::test]
async fn read_within_scope_succeeds() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("hello.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());
    let content = fs.read_to_string(Path::new("hello.txt")).await.unwrap();
    assert_eq!(content, "hello world");
}

#[tokio::test]
async fn read_outside_scope_fails() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let result = fs.read_to_string(Path::new("/etc/passwd")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn write_within_scope_succeeds() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    fs.write(Path::new("output.txt"), b"written data").await.unwrap();

    let content = std::fs::read_to_string(tmp.path().join("output.txt")).unwrap();
    assert_eq!(content, "written data");
}

#[tokio::test]
async fn path_traversal_attack_blocked() {
    let tmp = TempDir::new().unwrap();
    let fs = ScopedFs::new_with_root(tmp.path().to_path_buf());

    let malicious = Path::new("../../etc/passwd");
    let result = fs.read_to_string(malicious).await;
    assert!(result.is_err());
}
