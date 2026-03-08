pub mod crypto;
pub mod manifest;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

// Re-export key types for convenience.
pub use crypto::{EncryptedKeyStore, UnlockedKeys};
pub use state::{AppState, VaultStatus};
