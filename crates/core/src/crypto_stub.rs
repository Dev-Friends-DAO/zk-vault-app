//! Stub crypto module for wasm32 targets.
//!
//! Provides the same public API as `crypto.rs` but without native C FFI deps.
//! On wasm, real crypto operations return errors — the web UI is for display only.
//! Actual key management requires the native desktop/CLI build.

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{AppError, Result};

// ── Sensitive wrapper (same as native) ──

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SensitiveBytes32([u8; 32]);

impl SensitiveBytes32 {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(slice);
            Some(Self(arr))
        } else {
            None
        }
    }
}

// ── Key Store (same serialization types) ──

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EncryptedKeyStore {
    pub version: u32,
    pub kdf_salt: String,
    pub encrypted_master_key: String,
    pub master_key_nonce: String,
    pub encrypted_kem_sk: String,
    pub kem_sk_nonce: String,
    pub kem_pk: String,
    pub encrypted_x25519_sk: String,
    pub x25519_sk_nonce: String,
    pub x25519_pk: String,
    pub encrypted_mldsa_sk: String,
    pub mldsa_sk_nonce: String,
    pub mldsa_pk: String,
    pub encrypted_ed25519_sk: String,
    pub ed25519_sk_nonce: String,
    pub ed25519_pk: String,
}

impl EncryptedKeyStore {
    pub const CURRENT_VERSION: u32 = 1;
}

/// Decrypted key material (stub — keys are empty on wasm).
pub struct UnlockedKeys {
    pub master_key: SensitiveBytes32,
    pub kem_sk: Vec<u8>,
    pub kem_pk: Vec<u8>,
    pub x25519_sk: Vec<u8>,
    pub x25519_pk: [u8; 32],
    pub mldsa_sk: Vec<u8>,
    pub mldsa_pk: Vec<u8>,
    pub ed25519_sk: [u8; 32],
    pub ed25519_pk: [u8; 32],
}

impl Drop for UnlockedKeys {
    fn drop(&mut self) {
        self.kem_sk.zeroize();
        self.x25519_sk.zeroize();
        self.mldsa_sk.zeroize();
        self.ed25519_sk.zeroize();
    }
}

// ── Stub operations ──

pub fn generate_key_store(_passphrase: &[u8]) -> Result<EncryptedKeyStore> {
    Err(AppError::Crypto(
        "Key generation is not available in the web build. Use the desktop app or CLI.".into(),
    ))
}

pub fn unlock_all_keys(_passphrase: &[u8], _store: &EncryptedKeyStore) -> Result<UnlockedKeys> {
    Err(AppError::Crypto(
        "Vault unlock is not available in the web build. Use the desktop app or CLI.".into(),
    ))
}

pub fn unlock_master_key(_passphrase: &[u8], _store: &EncryptedKeyStore) -> Result<SensitiveBytes32> {
    Err(AppError::Crypto(
        "Vault unlock is not available in the web build. Use the desktop app or CLI.".into(),
    ))
}

// ── Vault path helpers ──

pub fn vault_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".zk-vault")
}

pub fn keystore_path() -> std::path::PathBuf {
    vault_dir().join("keystore.json")
}

pub fn save_key_store(_store: &EncryptedKeyStore) -> Result<()> {
    Err(AppError::Crypto(
        "Saving keystore is not available in the web build.".into(),
    ))
}

pub fn load_key_store() -> Result<EncryptedKeyStore> {
    Err(AppError::Crypto(
        "Loading keystore is not available in the web build.".into(),
    ))
}

/// BLAKE3 fingerprint stub — uses simple hash on wasm.
pub fn fingerprint(pk_bytes: &[u8]) -> String {
    // Simple hash for display purposes only
    let mut hash = [0u8; 8];
    for (i, &b) in pk_bytes.iter().enumerate() {
        hash[i % 8] ^= b;
    }
    hex::encode(hash)
}
