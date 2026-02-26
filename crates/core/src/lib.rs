pub mod api;
pub mod auth;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("API error: {0}")]
    Api(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Auth error: {0}")]
    Auth(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
