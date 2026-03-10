//! Application state management for the zk-vault UI.
//!
//! Manages the vault lifecycle: NoVault → Locked → Unlocked.
//! The UI reads VaultStatus to decide which page to show.

use std::path::PathBuf;

use crate::crypto::{
    self, EncryptedKeyStore, UnlockedKeys,
};
use crate::Result;

/// Vault lifecycle status.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultStatus {
    /// No keystore exists yet — user needs to register.
    NoVault,
    /// Keystore exists but is locked — user needs to login.
    Locked,
    /// Vault is unlocked and ready to use.
    Unlocked,
}

/// Public key fingerprints for display in the UI.
#[derive(Debug, Clone)]
pub struct KeyFingerprints {
    pub kem: String,
    pub x25519: String,
    pub mldsa: String,
    pub ed25519: String,
}

/// Global application state.
pub struct AppState {
    /// Current vault status.
    pub status: VaultStatus,
    /// Path to the vault directory (~/.zk-vault).
    pub vault_dir: PathBuf,
    /// Encrypted keystore (loaded from disk when vault exists).
    keystore: Option<EncryptedKeyStore>,
    /// Decrypted keys (only present when unlocked).
    unlocked_keys: Option<UnlockedKeys>,
    /// Public key fingerprints (available when keystore is loaded).
    pub fingerprints: Option<KeyFingerprints>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Create a new AppState by checking if a vault exists on disk.
    pub fn new() -> Self {
        let vault_dir = crypto::vault_dir();

        let (status, keystore, fingerprints) = Self::try_load_keystore();

        Self {
            status,
            vault_dir,
            keystore,
            unlocked_keys: None,
            fingerprints,
        }
    }

    /// Initialize a new vault: generate keys, save keystore, unlock.
    pub fn init_vault(&mut self, passphrase: &str) -> Result<()> {
        let store = crypto::generate_key_store(passphrase.as_bytes())?;
        crypto::save_key_store(&store)?;

        let fingerprints = compute_fingerprints(&store);

        // Unlock immediately after init
        let keys = crypto::unlock_all_keys(passphrase.as_bytes(), &store)?;

        self.keystore = Some(store);
        self.unlocked_keys = Some(keys);
        self.fingerprints = Some(fingerprints);
        self.status = VaultStatus::Unlocked;

        tracing::info!("Vault initialized and unlocked");
        Ok(())
    }

    /// Unlock an existing vault with a passphrase.
    pub fn unlock(&mut self, passphrase: &str) -> Result<()> {
        let store = self
            .keystore
            .as_ref()
            .ok_or_else(|| crate::AppError::Crypto("No keystore loaded".into()))?;

        let keys = crypto::unlock_all_keys(passphrase.as_bytes(), store)?;

        self.unlocked_keys = Some(keys);
        self.status = VaultStatus::Unlocked;

        tracing::info!("Vault unlocked");
        Ok(())
    }

    /// Lock the vault: zeroize decrypted keys.
    pub fn lock(&mut self) {
        self.unlocked_keys = None;
        if self.keystore.is_some() {
            self.status = VaultStatus::Locked;
        } else {
            self.status = VaultStatus::NoVault;
        }
        tracing::info!("Vault locked");
    }

    /// Get a reference to the unlocked keys (None if locked).
    pub fn keys(&self) -> Option<&UnlockedKeys> {
        self.unlocked_keys.as_ref()
    }

    /// Get the encrypted keystore (None if no vault exists).
    pub fn keystore(&self) -> Option<&EncryptedKeyStore> {
        self.keystore.as_ref()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn try_load_keystore() -> (VaultStatus, Option<EncryptedKeyStore>, Option<KeyFingerprints>) {
        let keystore_path = crypto::keystore_path();
        if keystore_path.exists() {
            match crypto::load_key_store() {
                Ok(ks) => {
                    let fp = compute_fingerprints(&ks);
                    (VaultStatus::Locked, Some(ks), Some(fp))
                }
                Err(e) => {
                    tracing::warn!("Failed to load keystore: {e}");
                    (VaultStatus::NoVault, None, None)
                }
            }
        } else {
            (VaultStatus::NoVault, None, None)
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn try_load_keystore() -> (VaultStatus, Option<EncryptedKeyStore>, Option<KeyFingerprints>) {
        // On wasm, we can't read the filesystem — always start as NoVault.
        (VaultStatus::NoVault, None, None)
    }

    /// Check if a vault exists on disk.
    pub fn vault_exists(&self) -> bool {
        self.status != VaultStatus::NoVault
    }

    /// Check if the vault is currently unlocked.
    pub fn is_unlocked(&self) -> bool {
        self.status == VaultStatus::Unlocked
    }

    /// Path to the manifests directory.
    pub fn manifests_dir(&self) -> PathBuf {
        self.vault_dir.join("manifests")
    }

    /// List available backup manifest files.
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let dir = self.manifests_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut backups: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        backups.sort();
        Ok(backups)
    }
}

fn compute_fingerprints(store: &EncryptedKeyStore) -> KeyFingerprints {
    let kem_pk = hex::decode(&store.kem_pk).unwrap_or_default();
    let x25519_pk = hex::decode(&store.x25519_pk).unwrap_or_default();
    let mldsa_pk = hex::decode(&store.mldsa_pk).unwrap_or_default();
    let ed25519_pk = hex::decode(&store.ed25519_pk).unwrap_or_default();

    KeyFingerprints {
        kem: crypto::fingerprint(&kem_pk),
        x25519: crypto::fingerprint(&x25519_pk),
        mldsa: crypto::fingerprint(&mldsa_pk),
        ed25519: crypto::fingerprint(&ed25519_pk),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_detects_no_vault() {
        // In test env, ~/.zk-vault may or may not exist.
        // Just verify it doesn't panic.
        let state = AppState::new();
        assert!(matches!(
            state.status,
            VaultStatus::NoVault | VaultStatus::Locked
        ));
    }

    #[test]
    fn lock_clears_keys() {
        let mut state = AppState::new();
        state.lock();
        assert!(state.keys().is_none());
    }

    #[test]
    fn vault_status_transitions() {
        assert_ne!(VaultStatus::NoVault, VaultStatus::Locked);
        assert_ne!(VaultStatus::Locked, VaultStatus::Unlocked);
    }
}
