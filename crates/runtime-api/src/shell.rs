use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ShellError(#[from] std::io::Error);

pub struct ShellApi;

impl ShellApi {
    pub fn open_url(url: &str) -> Result<(), ShellError> {
        Ok(open::that(url)?)
    }

    pub fn open_path(path: &Path) -> Result<(), ShellError> {
        Ok(open::that(path)?)
    }
}
