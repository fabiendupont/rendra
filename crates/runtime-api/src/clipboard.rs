use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ClipboardError(#[from] arboard::Error);

pub struct ClipboardApi;

impl ClipboardApi {
    pub fn read_text() -> Result<String, ClipboardError> {
        let mut clipboard = arboard::Clipboard::new()?;
        Ok(clipboard.get_text()?)
    }

    pub fn write_text(text: &str) -> Result<(), ClipboardError> {
        let mut clipboard = arboard::Clipboard::new()?;
        Ok(clipboard.set_text(text)?)
    }
}
