//! Configuration management for zk-vault.
//!
//! Reads/writes `~/.zk-vault/config.toml` — same format as the CLI.

use serde::{Deserialize, Serialize};

use crate::crypto;
use crate::Result;

/// Top-level config file structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaultConfig {
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageConfig {
    pub s3: Option<S3Config>,
}

/// S3-compatible storage configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    #[serde(default)]
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub path_style: bool,
}

impl S3Config {
    pub fn is_configured(&self) -> bool {
        !self.bucket.is_empty() && !self.access_key.is_empty() && !self.secret_key.is_empty()
    }
}

fn config_path() -> std::path::PathBuf {
    crypto::vault_dir().join("config.toml")
}

/// Load config from `~/.zk-vault/config.toml`. Returns default if file doesn't exist.
pub fn load_config() -> Result<VaultConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(VaultConfig::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    toml::from_str(&contents).map_err(|e| crate::AppError::Crypto(format!("Config parse error: {e}")))
}

/// Save config to `~/.zk-vault/config.toml`.
pub fn save_config(config: &VaultConfig) -> Result<()> {
    let dir = crypto::vault_dir();
    std::fs::create_dir_all(&dir)?;

    let path = config_path();
    let contents =
        toml::to_string_pretty(config).map_err(|e| crate::AppError::Crypto(format!("Config serialize error: {e}")))?;
    std::fs::write(&path, contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s3_config_is_configured() {
        let empty = S3Config::default();
        assert!(!empty.is_configured());

        let configured = S3Config {
            bucket: "my-bucket".into(),
            region: "us-east-1".into(),
            endpoint: None,
            access_key: "AKIA...".into(),
            secret_key: "secret".into(),
            path_style: false,
        };
        assert!(configured.is_configured());
    }

    #[test]
    fn config_roundtrip_toml() {
        let config = VaultConfig {
            storage: StorageConfig {
                s3: Some(S3Config {
                    bucket: "test-bucket".into(),
                    region: "eu-west-1".into(),
                    endpoint: Some("https://s3.example.com".into()),
                    access_key: "key".into(),
                    secret_key: "secret".into(),
                    path_style: true,
                }),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: VaultConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.storage.s3.unwrap().bucket, "test-bucket");
    }

    #[test]
    fn config_default_no_s3() {
        let config = VaultConfig::default();
        assert!(config.storage.s3.is_none());
    }
}
