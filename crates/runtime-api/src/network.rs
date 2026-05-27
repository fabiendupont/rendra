use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("HTTP is not allowed; use HTTPS")]
    HttpNotAllowed,
}

pub struct NetworkApi;

impl NetworkApi {
    pub async fn fetch(url: &str) -> Result<String, NetworkError> {
        if url.starts_with("http://") {
            return Err(NetworkError::HttpNotAllowed);
        }
        let response = reqwest::get(url).await?;
        Ok(response.text().await?)
    }

    pub async fn fetch_json(url: &str) -> Result<serde_json::Value, NetworkError> {
        if url.starts_with("http://") {
            return Err(NetworkError::HttpNotAllowed);
        }
        let response = reqwest::get(url).await?;
        Ok(response.json().await?)
    }
}
