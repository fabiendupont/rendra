use std::collections::BTreeSet;
use std::path::Path;

use runtime_sandbox::manifest::AppManifest;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("manifest error: {0}")]
    Manifest(#[from] runtime_sandbox::manifest::ManifestError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no app.toml found")]
    NoManifest,
}

pub struct CheckResult {
    pub declared: BTreeSet<String>,
    pub used: BTreeSet<String>,
}

impl CheckResult {
    pub fn over_permissioned(&self) -> BTreeSet<&str> {
        self.declared
            .iter()
            .filter(|p| !self.used.contains(p.as_str()))
            .map(|s| s.as_str())
            .collect()
    }

    pub fn under_permissioned(&self) -> BTreeSet<&str> {
        self.used
            .iter()
            .filter(|p| !self.declared.contains(p.as_str()))
            .map(|s| s.as_str())
            .collect()
    }

    pub fn is_ok(&self) -> bool {
        self.under_permissioned().is_empty()
    }
}

const API_PATTERNS: &[(&str, &str)] = &[
    ("runtime_api::filesystem", "filesystem"),
    ("runtime_api::network", "network"),
    ("runtime_api::clipboard", "clipboard"),
    ("runtime_api::dialog", "filesystem"),
    ("runtime_api::shell", "shell"),
    ("ScopedFs", "filesystem"),
    ("NetworkApi", "network"),
    ("ClipboardApi", "clipboard"),
    ("DialogApi", "filesystem"),
    ("ShellApi", "shell"),
];

pub fn run_check(project_dir: &Path) -> Result<CheckResult, CheckError> {
    let manifest_path = project_dir.join("app.toml");
    if !manifest_path.exists() {
        return Err(CheckError::NoManifest);
    }

    let manifest = AppManifest::from_file(&manifest_path)?;

    let mut declared = BTreeSet::new();
    if manifest.has_permission("network") {
        declared.insert("network".to_string());
    }
    if manifest.has_permission("filesystem") {
        declared.insert("filesystem".to_string());
    }
    if manifest.has_permission("clipboard") {
        declared.insert("clipboard".to_string());
    }

    let src_dir = project_dir.join("src");
    let mut used = BTreeSet::new();

    if src_dir.exists() {
        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for (pattern, feature) in API_PATTERNS {
                    if content.contains(pattern) {
                        used.insert(feature.to_string());
                    }
                }
            }
        }
    }

    Ok(CheckResult { declared, used })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project(toml_content: &str, rust_content: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("app.toml"), toml_content).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src").join("main.rs"), rust_content).unwrap();
        tmp
    }

    #[test]
    fn detects_over_permissioned() {
        let tmp = setup_project(
            r#"
[app]
name = "test"
version = "0.1.0"
[permissions]
network = ["https"]
clipboard = ["read"]
"#,
            "fn main() {}",
        );
        let result = run_check(tmp.path()).unwrap();
        assert!(result.over_permissioned().contains("network"));
        assert!(result.over_permissioned().contains("clipboard"));
        assert!(result.under_permissioned().is_empty());
    }

    #[test]
    fn detects_under_permissioned() {
        let tmp = setup_project(
            r#"
[app]
name = "test"
version = "0.1.0"
"#,
            r#"
use runtime_api::network::NetworkApi;
fn main() { NetworkApi::fetch("https://example.com"); }
"#,
        );
        let result = run_check(tmp.path()).unwrap();
        assert!(result.under_permissioned().contains("network"));
        assert!(result.declared.is_empty());
    }

    #[test]
    fn passes_when_matched() {
        let tmp = setup_project(
            r#"
[app]
name = "test"
version = "0.1.0"
[permissions]
network = ["https"]
"#,
            r#"
use runtime_api::network::NetworkApi;
fn main() { NetworkApi::fetch("https://example.com"); }
"#,
        );
        let result = run_check(tmp.path()).unwrap();
        assert!(result.is_ok());
        assert!(result.over_permissioned().is_empty());
        assert!(result.under_permissioned().is_empty());
    }
}
