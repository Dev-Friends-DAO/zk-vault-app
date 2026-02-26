use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, Result};

/// API client for the zk-vault backend.
/// All crypto operations happen client-side; this only sends ciphertext.
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

// ── Request / Response types ──

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub opaque_registration: Vec<u8>,
    pub encrypted_key_store: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub user_id: Uuid,
    pub opaque_login_proof: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub encrypted_key_store: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct CreateBackupRequest {
    pub source_type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackupJob {
    pub job_id: Uuid,
    pub status: String,
    pub files_processed: u64,
    pub bytes_uploaded: u64,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectSourceRequest {
    pub source_type: String,
    pub encrypted_tokens: Vec<u8>,
    pub token_nonce: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourceInfo {
    pub id: Uuid,
    pub source_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

// ── Implementation ──

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_header(&self) -> Result<String> {
        self.token
            .as_ref()
            .map(|t| format!("Bearer {t}"))
            .ok_or_else(|| AppError::Auth("Not authenticated".into()))
    }

    // ── Public endpoints ──

    pub async fn health(&self) -> Result<HealthResponse> {
        let resp = self.client.get(self.url("/health")).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn register(&self, req: &RegisterRequest) -> Result<RegisterResponse> {
        let resp = self
            .client
            .post(self.url("/api/auth/register"))
            .json(req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Api(err.error));
        }
        Ok(resp.json().await?)
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse> {
        let resp = self
            .client
            .post(self.url("/api/auth/login"))
            .json(req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Auth(err.error));
        }
        Ok(resp.json().await?)
    }

    // ── Protected endpoints ──

    pub async fn create_backup(&self, source_type: &str) -> Result<BackupJob> {
        let auth = self.auth_header()?;
        let resp = self
            .client
            .post(self.url("/api/backups"))
            .header("Authorization", auth)
            .json(&CreateBackupRequest {
                source_type: source_type.to_string(),
            })
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Api(err.error));
        }
        Ok(resp.json().await?)
    }

    pub async fn list_backups(&self) -> Result<Vec<BackupJob>> {
        let auth = self.auth_header()?;
        let resp = self
            .client
            .get(self.url("/api/backups"))
            .header("Authorization", auth)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Api(err.error));
        }
        Ok(resp.json().await?)
    }

    pub async fn connect_source(&self, req: &ConnectSourceRequest) -> Result<SourceInfo> {
        let auth = self.auth_header()?;
        let resp = self
            .client
            .post(self.url("/api/sources"))
            .header("Authorization", auth)
            .json(req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Api(err.error));
        }
        Ok(resp.json().await?)
    }

    pub async fn list_sources(&self) -> Result<Vec<SourceInfo>> {
        let auth = self.auth_header()?;
        let resp = self
            .client
            .get(self.url("/api/sources"))
            .header("Authorization", auth)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().await?;
            return Err(AppError::Api(err.error));
        }
        Ok(resp.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_client_default_unauthenticated() {
        let client = ApiClient::new("http://localhost:3000");
        assert!(!client.is_authenticated());
    }

    #[test]
    fn api_client_set_token() {
        let mut client = ApiClient::new("http://localhost:3000");
        client.set_token("test-jwt".into());
        assert!(client.is_authenticated());
        client.clear_token();
        assert!(!client.is_authenticated());
    }
}
