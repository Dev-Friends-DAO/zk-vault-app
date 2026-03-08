//! Backup manifest types — compatible with zk-vault CLI format.
//!
//! These types mirror `zk_vault::manifest` so the app can read
//! manifests created by the CLI and vice versa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u8,
    pub backup_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub source: String,
    pub files: Vec<ManifestFileEntry>,
    pub merkle_root: [u8; 32],
    pub anchor_receipts: Vec<AnchorReceipt>,
    pub total_original_size: u64,
    pub total_encrypted_size: u64,
    pub file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    pub source_path: String,
    pub source_id: String,
    pub content_hash: [u8; 32],
    pub original_size: u64,
    pub encrypted_size: u64,
    pub mime_type: Option<String>,
    pub source_modified_at: Option<DateTime<Utc>>,
    pub storage_locations: Vec<StorageLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    pub backend: String,
    pub storage_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorReceipt {
    pub chain: String,
    pub tx_id: String,
    pub anchored_hash: [u8; 32],
    pub block_number: Option<u64>,
    pub fee: Option<String>,
}

impl BackupManifest {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

/// Summary of a backup for dashboard display.
#[derive(Debug, Clone, PartialEq)]
pub struct BackupSummary {
    pub backup_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub source: String,
    pub file_count: u32,
    pub total_original_size: u64,
    pub total_encrypted_size: u64,
    pub merkle_root: String,
    pub anchored: bool,
}

impl From<&BackupManifest> for BackupSummary {
    fn from(m: &BackupManifest) -> Self {
        Self {
            backup_id: m.backup_id,
            created_at: m.created_at,
            source: m.source.clone(),
            file_count: m.file_count,
            total_original_size: m.total_original_size,
            total_encrypted_size: m.total_encrypted_size,
            merkle_root: hex::encode(&m.merkle_root[..8]),
            anchored: !m.anchor_receipts.is_empty(),
        }
    }
}

/// Load all backup summaries from the manifests directory.
pub fn load_backup_summaries(manifests_dir: &std::path::Path) -> Result<Vec<BackupSummary>> {
    if !manifests_dir.exists() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    for entry in std::fs::read_dir(manifests_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            match std::fs::read(&path) {
                Ok(data) => match BackupManifest::from_bytes(&data) {
                    Ok(manifest) => summaries.push(BackupSummary::from(&manifest)),
                    Err(e) => tracing::warn!("Failed to parse manifest {}: {e}", path.display()),
                },
                Err(e) => tracing::warn!("Failed to read manifest {}: {e}", path.display()),
            }
        }
    }

    // Sort by creation time, newest first
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(summaries)
}

/// Format bytes into human-readable size.
pub fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_formatting() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1024), "1.0 KB");
        assert_eq!(human_size(1_048_576), "1.0 MB");
        assert_eq!(human_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn backup_summary_from_manifest() {
        let manifest = BackupManifest {
            version: 1,
            backup_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            created_at: Utc::now(),
            source: "local".into(),
            files: vec![],
            merkle_root: [0xAB; 32],
            anchor_receipts: vec![],
            total_original_size: 1024,
            total_encrypted_size: 1200,
            file_count: 3,
        };

        let summary = BackupSummary::from(&manifest);
        assert_eq!(summary.file_count, 3);
        assert_eq!(summary.total_original_size, 1024);
        assert!(!summary.anchored);
        assert_eq!(summary.merkle_root, "abababababababab");
    }
}
