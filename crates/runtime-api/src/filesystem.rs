use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("path is outside the allowed scope: {0}")]
    OutOfScope(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct ScopedFs {
    root: PathBuf,
}

impl ScopedFs {
    pub fn new_with_root(root: PathBuf) -> Self {
        Self { root }
    }

    fn validate_path(&self, path: &Path) -> Result<PathBuf, FsError> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        // Try to canonicalize the full path first (works for existing paths).
        if let Ok(canonical) = candidate.canonicalize() {
            let canonical_root = self.root.canonicalize().map_err(FsError::Io)?;
            if canonical.starts_with(&canonical_root) {
                return Ok(canonical);
            }
            return Err(FsError::OutOfScope(path.to_path_buf()));
        }

        // For non-existing paths (e.g. writes to new files), canonicalize the
        // parent directory and append the file name.
        if let Some(parent) = candidate.parent() {
            if let Ok(canonical_parent) = parent.canonicalize() {
                let canonical_root = self.root.canonicalize().map_err(FsError::Io)?;
                if let Some(file_name) = candidate.file_name() {
                    let full = canonical_parent.join(file_name);
                    if full.starts_with(&canonical_root) {
                        return Ok(full);
                    }
                }
            }
        }

        Err(FsError::OutOfScope(path.to_path_buf()))
    }

    pub async fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        let validated = self.validate_path(path)?;
        Ok(tokio::fs::read_to_string(validated).await?)
    }

    pub async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        let validated = self.validate_path(path)?;
        Ok(tokio::fs::read(validated).await?)
    }

    pub async fn write(&self, path: &Path, contents: &[u8]) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        Ok(tokio::fs::write(validated, contents).await?)
    }

    pub async fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        Ok(tokio::fs::remove_file(validated).await?)
    }

    pub async fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let validated = self.validate_path(path)?;
        Ok(tokio::fs::create_dir_all(validated).await?)
    }
}
