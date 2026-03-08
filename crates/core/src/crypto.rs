//! Client-side cryptographic operations.
//!
//! Uses the SAME primitives, domain separators, and AADs as the zk-vault CLI
//! to ensure keystore interoperability:
//!   - Argon2id (t=3, m=256MB, p=4) for KDF
//!   - XChaCha20-Poly1305 for AEAD
//!   - ML-KEM-768 (pqcrypto-kyber) + X25519 hybrid KEM
//!   - ML-DSA-65 + Ed25519 hybrid signatures
//!   - BLAKE3 for hashing and key combining

use argon2::Argon2;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{
    Ciphertext as _, PublicKey as PqPublicKey, SecretKey as PqSecretKey, SharedSecret as _,
};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey};
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{AppError, Result};

// ── Sensitive wrapper ──

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

// ── KDF: Argon2id ──

const ARGON2_T_COST: u32 = 3;
const ARGON2_M_COST: u32 = 262144; // 256 MiB
const ARGON2_P_COST: u32 = 4;

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<SensitiveBytes32> {
    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| AppError::Crypto(format!("Argon2 params: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut output)
        .map_err(|e| AppError::Crypto(format!("Argon2 derivation: {e}")))?;
    Ok(SensitiveBytes32::new(output))
}

/// Fast KDF for tests only (t=1, m=16KB, p=1).
#[cfg(test)]
pub fn derive_key_test(passphrase: &[u8], salt: &[u8]) -> Result<SensitiveBytes32> {
    let params = argon2::Params::new(16, 1, 1, Some(32))
        .map_err(|e| AppError::Crypto(format!("Argon2 params: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut output)
        .map_err(|e| AppError::Crypto(format!("Argon2 derivation: {e}")))?;
    Ok(SensitiveBytes32::new(output))
}

// ── AEAD: XChaCha20-Poly1305 ──

pub fn generate_symmetric_key() -> SensitiveBytes32 {
    let mut key = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut key);
    SensitiveBytes32::new(key)
}

pub fn generate_nonce() -> [u8; 24] {
    let mut nonce = [0u8; 24];
    use rand::RngCore;
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt(
    key: &SensitiveBytes32,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<([u8; 24], Vec<u8>)> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|e| AppError::Crypto(format!("AEAD key: {e}")))?;
    let nonce_bytes = generate_nonce();
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad })
        .map_err(|e| AppError::Crypto(format!("Encryption: {e}")))?;
    Ok((nonce_bytes, ciphertext))
}

pub fn decrypt(
    key: &SensitiveBytes32,
    nonce: &[u8; 24],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|e| AppError::Crypto(format!("AEAD key: {e}")))?;
    let n = XNonce::from_slice(nonce);
    cipher
        .decrypt(n, Payload { msg: ciphertext, aad })
        .map_err(|e| AppError::Crypto(format!("Decryption: {e}")))
}

pub fn encrypt_with_nonce(
    key: &SensitiveBytes32,
    nonce: &[u8; 24],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes())
        .map_err(|e| AppError::Crypto(format!("AEAD key: {e}")))?;
    let n = XNonce::from_slice(nonce);
    cipher
        .encrypt(n, Payload { msg: plaintext, aad })
        .map_err(|e| AppError::Crypto(format!("Encryption: {e}")))
}

// ── Hybrid KEM: ML-KEM-768 + X25519 ──
// Domain separators match zk-vault CLI exactly.

const KEM_DOMAIN_SEPARATOR: &[u8; 32] = b"zk-vault-hybrid-kem-v1-combine!!";
const KEY_WRAP_AAD: &[u8] = b"zk-vault-keywrap-v1";
const KEY_WRAP_NONCE: [u8; 24] = [0u8; 24];

pub struct KemKeyPair {
    pub public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

impl KemKeyPair {
    pub fn generate() -> Self {
        let (pk, sk) = kyber768::keypair();
        Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }

    pub fn secret_key_bytes(&self) -> &[u8] {
        &self.secret_key
    }
}

impl Drop for KemKeyPair {
    fn drop(&mut self) {
        self.secret_key.zeroize();
    }
}

pub struct X25519KeyPair {
    pub public_key: X25519PublicKey,
    secret_key: StaticSecret,
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        Self {
            public_key: public,
            secret_key: secret,
        }
    }

    pub fn secret_key(&self) -> &StaticSecret {
        &self.secret_key
    }
}

pub struct HybridPublicKey {
    pub kem_pk: Vec<u8>,
    pub x25519_pk: [u8; 32],
}

pub struct EncapsulationResult {
    pub kem_ciphertext: Vec<u8>,
    pub eph_x25519_pk: [u8; 32],
    pub wrapped_key: Vec<u8>,
}

fn combine_shared_secrets(ss_kem: &[u8], ss_x25519: &[u8]) -> SensitiveBytes32 {
    let mut combined = Vec::with_capacity(ss_kem.len() + ss_x25519.len());
    combined.extend_from_slice(ss_kem);
    combined.extend_from_slice(ss_x25519);

    let result = blake3::keyed_hash(KEM_DOMAIN_SEPARATOR, &combined);
    combined.zeroize();

    SensitiveBytes32::new(*result.as_bytes())
}

pub fn encapsulate(
    hybrid_pk: &HybridPublicKey,
    sym_key: &SensitiveBytes32,
) -> Result<EncapsulationResult> {
    // ML-KEM-768 encapsulation
    let kem_pk = kyber768::PublicKey::from_bytes(&hybrid_pk.kem_pk)
        .map_err(|e| AppError::Crypto(format!("Invalid ML-KEM-768 public key: {e:?}")))?;
    let (ss_kem, kem_ct) = kyber768::encapsulate(&kem_pk);

    // X25519 ephemeral DH
    let eph_secret = EphemeralSecret::random_from_rng(OsRng);
    let eph_public = X25519PublicKey::from(&eph_secret);
    let recipient_pk = X25519PublicKey::from(hybrid_pk.x25519_pk);
    let ss_x25519 = eph_secret.diffie_hellman(&recipient_pk);

    // Combine shared secrets
    let wrapping_key = combine_shared_secrets(ss_kem.as_bytes(), ss_x25519.as_bytes());

    // Wrap the symmetric key
    let wrapped_key =
        encrypt_with_nonce(&wrapping_key, &KEY_WRAP_NONCE, sym_key.as_bytes(), KEY_WRAP_AAD)?;

    Ok(EncapsulationResult {
        kem_ciphertext: kem_ct.as_bytes().to_vec(),
        eph_x25519_pk: eph_public.to_bytes(),
        wrapped_key,
    })
}

pub fn decapsulate(
    kem_sk: &[u8],
    x25519_sk: &StaticSecret,
    kem_ciphertext: &[u8],
    eph_x25519_pk: &[u8; 32],
    wrapped_key: &[u8],
) -> Result<SensitiveBytes32> {
    // ML-KEM-768 decapsulation
    let sk = kyber768::SecretKey::from_bytes(kem_sk)
        .map_err(|e| AppError::Crypto(format!("Invalid ML-KEM-768 secret key: {e:?}")))?;
    let ct = kyber768::Ciphertext::from_bytes(kem_ciphertext)
        .map_err(|e| AppError::Crypto(format!("Invalid ML-KEM-768 ciphertext: {e:?}")))?;
    let ss_kem = kyber768::decapsulate(&ct, &sk);

    // X25519 DH
    let eph_pk = X25519PublicKey::from(*eph_x25519_pk);
    let ss_x25519 = x25519_sk.diffie_hellman(&eph_pk);

    // Reconstruct wrapping key
    let wrapping_key = combine_shared_secrets(ss_kem.as_bytes(), ss_x25519.as_bytes());

    // Unwrap
    let sym_key_bytes = decrypt(&wrapping_key, &KEY_WRAP_NONCE, wrapped_key, KEY_WRAP_AAD)?;
    SensitiveBytes32::from_slice(&sym_key_bytes)
        .ok_or_else(|| AppError::Crypto("Unwrapped key is not 32 bytes".into()))
}

// ── Key Store ──
// Format and AADs match zk-vault CLI exactly for interoperability.

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

/// Decrypted key material. Zeroized on drop.
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

/// Generate a new key store from a passphrase.
pub fn generate_key_store(passphrase: &[u8]) -> Result<EncryptedKeyStore> {
    let salt = generate_salt();
    let pdk = derive_key(passphrase, &salt)?;
    generate_key_store_with_pdk(&pdk, &salt)
}

fn generate_key_store_with_pdk(
    pdk: &SensitiveBytes32,
    salt: &[u8],
) -> Result<EncryptedKeyStore> {
    let master_key = generate_symmetric_key();
    let kem_kp = KemKeyPair::generate();
    let x25519_kp = X25519KeyPair::generate();
    let (mldsa_pk, mldsa_sk) = dilithium3::keypair();
    let ed25519_sk = Ed25519SigningKey::generate(&mut OsRng);
    let ed25519_pk = ed25519_sk.verifying_key();

    // Encrypt each secret key with PDK — AADs match CLI exactly
    let (mk_nonce, mk_ct) = encrypt(pdk, master_key.as_bytes(), b"zk-vault:mk")?;
    let (kem_nonce, kem_ct) = encrypt(pdk, kem_kp.secret_key_bytes(), b"zk-vault:kem-sk")?;
    let (x25519_nonce, x25519_ct) = encrypt(
        pdk,
        x25519_kp.secret_key().to_bytes().as_ref(),
        b"zk-vault:x25519-sk",
    )?;
    let (mldsa_nonce, mldsa_ct) =
        encrypt(pdk, mldsa_sk.as_bytes(), b"zk-vault:mldsa-sk")?;
    let (ed25519_nonce, ed25519_ct) =
        encrypt(pdk, ed25519_sk.as_bytes(), b"zk-vault:ed25519-sk")?;

    Ok(EncryptedKeyStore {
        version: EncryptedKeyStore::CURRENT_VERSION,
        kdf_salt: hex::encode(salt),
        encrypted_master_key: hex::encode(mk_ct),
        master_key_nonce: hex::encode(mk_nonce),
        encrypted_kem_sk: hex::encode(kem_ct),
        kem_sk_nonce: hex::encode(kem_nonce),
        kem_pk: hex::encode(&kem_kp.public_key),
        encrypted_x25519_sk: hex::encode(x25519_ct),
        x25519_sk_nonce: hex::encode(x25519_nonce),
        x25519_pk: hex::encode(x25519_kp.public_key.as_bytes()),
        encrypted_mldsa_sk: hex::encode(mldsa_ct),
        mldsa_sk_nonce: hex::encode(mldsa_nonce),
        mldsa_pk: hex::encode(mldsa_pk.as_bytes()),
        encrypted_ed25519_sk: hex::encode(ed25519_ct),
        ed25519_sk_nonce: hex::encode(ed25519_nonce),
        ed25519_pk: hex::encode(ed25519_pk.as_bytes()),
    })
}

/// Decrypt a single field from the keystore.
fn decrypt_field(
    pdk: &SensitiveBytes32,
    nonce_hex: &str,
    ciphertext_hex: &str,
    aad: &[u8],
) -> Result<Vec<u8>> {
    let nonce_bytes = hex::decode(nonce_hex)
        .map_err(|e| AppError::Crypto(format!("Invalid nonce hex: {e}")))?;
    let nonce: [u8; 24] = nonce_bytes
        .try_into()
        .map_err(|_| AppError::Crypto("Invalid nonce length".into()))?;
    let ciphertext = hex::decode(ciphertext_hex)
        .map_err(|e| AppError::Crypto(format!("Invalid ciphertext hex: {e}")))?;
    decrypt(pdk, &nonce, &ciphertext, aad)
}

/// Unlock just the master key (fast path for login).
pub fn unlock_master_key(
    passphrase: &[u8],
    store: &EncryptedKeyStore,
) -> Result<SensitiveBytes32> {
    let salt = hex::decode(&store.kdf_salt)
        .map_err(|e| AppError::Crypto(format!("Invalid salt hex: {e}")))?;
    let pdk = derive_key(passphrase, &salt)?;

    let mk_bytes = decrypt_field(
        &pdk,
        &store.master_key_nonce,
        &store.encrypted_master_key,
        b"zk-vault:mk",
    )
    .map_err(|_| AppError::Crypto("Invalid passphrase".into()))?;

    SensitiveBytes32::from_slice(&mk_bytes)
        .ok_or_else(|| AppError::Crypto("Master key is not 32 bytes".into()))
}

/// Unlock all keys from the store.
pub fn unlock_all_keys(passphrase: &[u8], store: &EncryptedKeyStore) -> Result<UnlockedKeys> {
    let salt = hex::decode(&store.kdf_salt)
        .map_err(|e| AppError::Crypto(format!("Invalid salt hex: {e}")))?;
    let pdk = derive_key(passphrase, &salt)?;
    unlock_all_keys_with_pdk(&pdk, store)
}

fn unlock_all_keys_with_pdk(
    pdk: &SensitiveBytes32,
    store: &EncryptedKeyStore,
) -> Result<UnlockedKeys> {
    let mk_bytes = decrypt_field(
        pdk,
        &store.master_key_nonce,
        &store.encrypted_master_key,
        b"zk-vault:mk",
    )
    .map_err(|_| AppError::Crypto("Invalid passphrase".into()))?;
    let master_key = SensitiveBytes32::from_slice(&mk_bytes)
        .ok_or_else(|| AppError::Crypto("Master key is not 32 bytes".into()))?;

    let kem_sk = decrypt_field(pdk, &store.kem_sk_nonce, &store.encrypted_kem_sk, b"zk-vault:kem-sk")?;
    let x25519_sk = decrypt_field(
        pdk,
        &store.x25519_sk_nonce,
        &store.encrypted_x25519_sk,
        b"zk-vault:x25519-sk",
    )?;
    let mldsa_sk = decrypt_field(
        pdk,
        &store.mldsa_sk_nonce,
        &store.encrypted_mldsa_sk,
        b"zk-vault:mldsa-sk",
    )?;
    let ed25519_sk_bytes = decrypt_field(
        pdk,
        &store.ed25519_sk_nonce,
        &store.encrypted_ed25519_sk,
        b"zk-vault:ed25519-sk",
    )?;

    let kem_pk = hex::decode(&store.kem_pk)
        .map_err(|e| AppError::Crypto(format!("Invalid hex: {e}")))?;
    let x25519_pk_bytes = hex::decode(&store.x25519_pk)
        .map_err(|e| AppError::Crypto(format!("Invalid hex: {e}")))?;
    let mldsa_pk = hex::decode(&store.mldsa_pk)
        .map_err(|e| AppError::Crypto(format!("Invalid hex: {e}")))?;
    let ed25519_pk_bytes = hex::decode(&store.ed25519_pk)
        .map_err(|e| AppError::Crypto(format!("Invalid hex: {e}")))?;

    let x25519_pk: [u8; 32] = x25519_pk_bytes
        .try_into()
        .map_err(|_| AppError::Crypto("X25519 pk not 32 bytes".into()))?;
    let ed25519_sk: [u8; 32] = ed25519_sk_bytes
        .try_into()
        .map_err(|_| AppError::Crypto("Ed25519 sk not 32 bytes".into()))?;
    let ed25519_pk: [u8; 32] = ed25519_pk_bytes
        .try_into()
        .map_err(|_| AppError::Crypto("Ed25519 pk not 32 bytes".into()))?;

    Ok(UnlockedKeys {
        master_key,
        kem_sk,
        kem_pk,
        x25519_sk,
        x25519_pk,
        mldsa_sk,
        mldsa_pk,
        ed25519_sk,
        ed25519_pk,
    })
}

// ── Vault path helpers ──

pub fn vault_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".zk-vault")
}

pub fn keystore_path() -> std::path::PathBuf {
    vault_dir().join("keystore.json")
}

pub fn save_key_store(store: &EncryptedKeyStore) -> Result<()> {
    let dir = vault_dir();
    std::fs::create_dir_all(&dir)?;

    let path = keystore_path();
    let json = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load_key_store() -> Result<EncryptedKeyStore> {
    let path = keystore_path();
    let json = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&json)?)
}

/// BLAKE3 fingerprint of a public key (first 8 bytes, hex-encoded).
pub fn fingerprint(pk_bytes: &[u8]) -> String {
    let hash = blake3::hash(pk_bytes);
    hex::encode(&hash.as_bytes()[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_store_fast(passphrase: &[u8]) -> EncryptedKeyStore {
        let salt = generate_salt();
        let pdk = derive_key_test(passphrase, &salt).unwrap();
        generate_key_store_with_pdk(&pdk, &salt).unwrap()
    }

    #[test]
    fn aead_roundtrip() {
        let key = generate_symmetric_key();
        let plaintext = b"hello zk-vault";
        let aad = b"test-aad";
        let (nonce, ciphertext) = encrypt(&key, plaintext, aad).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn kdf_deterministic() {
        let salt = [42u8; 32];
        let k1 = derive_key_test(b"passphrase", &salt).unwrap();
        let k2 = derive_key_test(b"passphrase", &salt).unwrap();
        assert_eq!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn kdf_different_passphrase() {
        let salt = [42u8; 32];
        let k1 = derive_key_test(b"passphrase1", &salt).unwrap();
        let k2 = derive_key_test(b"passphrase2", &salt).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn key_store_roundtrip() {
        let passphrase = b"test-passphrase-12345";
        let store = generate_store_fast(passphrase);

        // Verify JSON serialization
        let json = serde_json::to_string(&store).unwrap();
        let loaded: EncryptedKeyStore = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.version, EncryptedKeyStore::CURRENT_VERSION);
        assert_eq!(loaded.kem_pk, store.kem_pk);

        // Verify unlock with fast KDF
        let salt = hex::decode(&store.kdf_salt).unwrap();
        let pdk = derive_key_test(passphrase, &salt).unwrap();
        let mk = decrypt_field(
            &pdk,
            &store.master_key_nonce,
            &store.encrypted_master_key,
            b"zk-vault:mk",
        )
        .unwrap();
        assert_eq!(mk.len(), 32);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let store = generate_store_fast(b"correct-passphrase");
        let salt = hex::decode(&store.kdf_salt).unwrap();
        let wrong_pdk = derive_key_test(b"wrong-passphrase", &salt).unwrap();
        let result = decrypt_field(
            &wrong_pdk,
            &store.master_key_nonce,
            &store.encrypted_master_key,
            b"zk-vault:mk",
        );
        assert!(result.is_err());
    }

    #[test]
    fn unlock_all_keys_roundtrip() {
        let passphrase = b"test-passphrase-12345";
        let store = generate_store_fast(passphrase);
        let salt = hex::decode(&store.kdf_salt).unwrap();
        let pdk = derive_key_test(passphrase, &salt).unwrap();

        let keys = unlock_all_keys_with_pdk(&pdk, &store).unwrap();
        assert_eq!(keys.master_key.as_bytes().len(), 32);
        assert!(!keys.kem_sk.is_empty());
        assert!(!keys.kem_pk.is_empty());
        assert_eq!(keys.x25519_sk.len(), 32);
        assert_eq!(keys.ed25519_sk.len(), 32);
        assert_eq!(keys.ed25519_pk.len(), 32);
    }

    #[test]
    fn kem_encapsulate_decapsulate() {
        let kem_kp = KemKeyPair::generate();
        let x25519_kp = X25519KeyPair::generate();
        let hybrid_pk = HybridPublicKey {
            kem_pk: kem_kp.public_key.clone(),
            x25519_pk: x25519_kp.public_key.to_bytes(),
        };

        let sym_key = generate_symmetric_key();
        let encap = encapsulate(&hybrid_pk, &sym_key).unwrap();

        let recovered = decapsulate(
            kem_kp.secret_key_bytes(),
            x25519_kp.secret_key(),
            &encap.kem_ciphertext,
            &encap.eph_x25519_pk,
            &encap.wrapped_key,
        )
        .unwrap();

        assert_eq!(sym_key.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn fingerprint_deterministic() {
        let data = b"test-public-key-data";
        assert_eq!(fingerprint(data), fingerprint(data));
    }
}
