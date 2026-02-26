//! Client-side cryptographic operations.
//! Same primitives as zk-vault backend: Argon2id, XChaCha20-Poly1305,
//! ML-KEM-768 + X25519 hybrid KEM, ML-DSA-65 + Ed25519 hybrid signatures.

use argon2::Argon2;
use blake3;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use ml_kem::{EncodedSizeUser, KemCore, MlKem768, MlKem768Params};
use ml_kem::kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey, SecretKey};
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::Result;
use crate::AppError;

// ── Sensitive wrappers ──

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
const ARGON2_M_COST: u32 = 262144; // 256 MiB in KiB
const ARGON2_P_COST: u32 = 4;

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<SensitiveBytes32> {
    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| AppError::Auth(format!("Argon2 params: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut output)
        .map_err(|e| AppError::Auth(format!("Argon2 derivation: {e}")))?;
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
    let cipher =
        XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|e| AppError::Auth(format!("AEAD key: {e}")))?;
    let nonce_bytes = generate_nonce();
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad })
        .map_err(|e| AppError::Auth(format!("Encryption: {e}")))?;
    Ok((nonce_bytes, ciphertext))
}

pub fn decrypt(
    key: &SensitiveBytes32,
    nonce: &[u8; 24],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|e| AppError::Auth(format!("AEAD key: {e}")))?;
    let n = XNonce::from_slice(nonce);
    cipher
        .decrypt(n, Payload { msg: ciphertext, aad })
        .map_err(|e| AppError::Auth(format!("Decryption: {e}")))
}

pub fn encrypt_with_nonce(
    key: &SensitiveBytes32,
    nonce: &[u8; 24],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|e| AppError::Auth(format!("AEAD key: {e}")))?;
    let n = XNonce::from_slice(nonce);
    cipher
        .encrypt(n, Payload { msg: plaintext, aad })
        .map_err(|e| AppError::Auth(format!("Encryption: {e}")))
}

// ── Hybrid KEM: ML-KEM-768 + X25519 ──

const KEM_DOMAIN_SEPARATOR: &[u8; 32] = b"zk-vault-kem-domain-sep-v1\x00\x00\x00\x00\x00\x00";
const KEM_WRAP_AAD: &[u8] = b"zk-vault-keywrap-v1";

pub struct KemKeyPair {
    pub public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

impl KemKeyPair {
    pub fn generate() -> Self {
        let (dk, ek) = MlKem768::generate(&mut OsRng);
        Self {
            public_key: ek.as_bytes().as_slice().to_vec(),
            secret_key: dk.as_bytes().as_slice().to_vec(),
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

pub fn encapsulate(
    hybrid_pk: &HybridPublicKey,
    sym_key: &SensitiveBytes32,
) -> Result<EncapsulationResult> {
    // ML-KEM encapsulate
    let pk_array = hybrid_pk.kem_pk.as_slice().try_into().map_err(|_| {
        AppError::Auth("Invalid ML-KEM public key length".into())
    })?;
    let ek = EncapsulationKey::<MlKem768Params>::from_bytes(pk_array);
    let (kem_ct, ss_kem) = ek.encapsulate(&mut OsRng).map_err(|_| {
        AppError::Auth("ML-KEM encapsulation failed".into())
    })?;

    // X25519 ephemeral DH
    let eph_secret = EphemeralSecret::random_from_rng(OsRng);
    let eph_public = X25519PublicKey::from(&eph_secret);
    let recipient_pk = X25519PublicKey::from(hybrid_pk.x25519_pk);
    let ss_x25519 = eph_secret.diffie_hellman(&recipient_pk);

    // Combine shared secrets
    let mut combined = Vec::new();
    combined.extend_from_slice(ss_kem.as_slice());
    combined.extend_from_slice(ss_x25519.as_bytes());
    let wrapping_key_bytes = blake3::keyed_hash(KEM_DOMAIN_SEPARATOR, &combined);
    let wrapping_key = SensitiveBytes32::new(*wrapping_key_bytes.as_bytes());

    // Wrap symmetric key
    let zero_nonce = [0u8; 24];
    let wrapped = encrypt_with_nonce(&wrapping_key, &zero_nonce, sym_key.as_bytes(), KEM_WRAP_AAD)?;

    Ok(EncapsulationResult {
        kem_ciphertext: kem_ct.as_slice().to_vec(),
        eph_x25519_pk: eph_public.to_bytes(),
        wrapped_key: wrapped,
    })
}

pub fn decapsulate(
    kem_sk: &[u8],
    x25519_sk: &StaticSecret,
    kem_ciphertext: &[u8],
    eph_x25519_pk: &[u8; 32],
    wrapped_key: &[u8],
) -> Result<SensitiveBytes32> {
    // ML-KEM decapsulate
    let sk_array = kem_sk.try_into().map_err(|_| {
        AppError::Auth("Invalid ML-KEM secret key length".into())
    })?;
    let dk = DecapsulationKey::<MlKem768Params>::from_bytes(sk_array);

    let ct = ml_kem::Ciphertext::<MlKem768>::try_from(kem_ciphertext).map_err(|_| {
        AppError::Auth("Invalid ML-KEM ciphertext length".into())
    })?;
    let ss_kem = dk.decapsulate(&ct).map_err(|_| {
        AppError::Auth("ML-KEM decapsulation failed".into())
    })?;

    // X25519 DH
    let eph_pk = X25519PublicKey::from(*eph_x25519_pk);
    let ss_x25519 = x25519_sk.diffie_hellman(&eph_pk);

    // Combine
    let mut combined = Vec::new();
    combined.extend_from_slice(ss_kem.as_slice());
    combined.extend_from_slice(ss_x25519.as_bytes());
    let wrapping_key_bytes = blake3::keyed_hash(KEM_DOMAIN_SEPARATOR, &combined);
    let wrapping_key = SensitiveBytes32::new(*wrapping_key_bytes.as_bytes());

    // Unwrap
    let zero_nonce = [0u8; 24];
    let sym_key_bytes = decrypt(&wrapping_key, &zero_nonce, wrapped_key, KEM_WRAP_AAD)?;
    SensitiveBytes32::from_slice(&sym_key_bytes)
        .ok_or_else(|| AppError::Auth("Decapsulated key wrong length".into()))
}

// ── Key Store: generation + encryption ──

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EncryptedKeyStore {
    pub version: u32,
    pub kdf_salt: String,
    pub encrypted_master_key: String,
    pub master_key_nonce: String,
    pub kem_pk: String,
    pub encrypted_kem_sk: String,
    pub kem_sk_nonce: String,
    pub x25519_pk: String,
    pub encrypted_x25519_sk: String,
    pub x25519_sk_nonce: String,
    pub mldsa_pk: String,
    pub encrypted_mldsa_sk: String,
    pub mldsa_sk_nonce: String,
    pub ed25519_pk: String,
    pub encrypted_ed25519_sk: String,
    pub ed25519_sk_nonce: String,
}

/// Generate all keys and produce an encrypted key store from a passphrase.
/// Returns (EncryptedKeyStore as JSON bytes, OPAQUE-like registration blob).
pub fn generate_key_store(passphrase: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    // 1. KDF
    let salt = generate_salt();
    let derived_key = derive_key(passphrase.as_bytes(), &salt)?;

    // 2. Master key
    let master_key = generate_symmetric_key();
    let (mk_nonce, encrypted_mk) = encrypt(&derived_key, master_key.as_bytes(), b"master-key")?;

    // 3. ML-KEM-768
    let kem_kp = KemKeyPair::generate();
    let (kem_nonce, encrypted_kem_sk) = encrypt(&derived_key, kem_kp.secret_key_bytes(), b"kem-sk")?;

    // 4. X25519
    let x25519_kp = X25519KeyPair::generate();
    let (x25519_nonce, encrypted_x25519_sk) =
        encrypt(&derived_key, x25519_kp.secret_key().as_bytes(), b"x25519-sk")?;

    // 5. ML-DSA-65
    let (mldsa_pk, mldsa_sk) = dilithium3::keypair();
    let mldsa_pk_bytes = mldsa_pk.as_bytes().to_vec();
    let mldsa_sk_bytes = mldsa_sk.as_bytes().to_vec();
    let (mldsa_nonce, encrypted_mldsa_sk) = encrypt(&derived_key, &mldsa_sk_bytes, b"mldsa-sk")?;

    // 6. Ed25519
    let ed25519_sk = Ed25519SigningKey::generate(&mut OsRng);
    let ed25519_pk = ed25519_sk.verifying_key();
    let (ed25519_nonce, encrypted_ed25519_sk) =
        encrypt(&derived_key, ed25519_sk.as_bytes(), b"ed25519-sk")?;

    let key_store = EncryptedKeyStore {
        version: 1,
        kdf_salt: hex::encode(salt),
        encrypted_master_key: hex::encode(&encrypted_mk),
        master_key_nonce: hex::encode(mk_nonce),
        kem_pk: hex::encode(&kem_kp.public_key),
        encrypted_kem_sk: hex::encode(&encrypted_kem_sk),
        kem_sk_nonce: hex::encode(kem_nonce),
        x25519_pk: hex::encode(x25519_kp.public_key.as_bytes()),
        encrypted_x25519_sk: hex::encode(&encrypted_x25519_sk),
        x25519_sk_nonce: hex::encode(x25519_nonce),
        mldsa_pk: hex::encode(&mldsa_pk_bytes),
        encrypted_mldsa_sk: hex::encode(&encrypted_mldsa_sk),
        mldsa_sk_nonce: hex::encode(mldsa_nonce),
        ed25519_pk: hex::encode(ed25519_pk.as_bytes()),
        encrypted_ed25519_sk: hex::encode(&encrypted_ed25519_sk),
        ed25519_sk_nonce: hex::encode(ed25519_nonce),
    };

    let key_store_json = serde_json::to_vec(&key_store)?;

    // OPAQUE registration blob: for now, hash(derived_key) as a placeholder
    // Full OPAQUE (opaque-ke) will replace this
    let opaque_blob = blake3::hash(derived_key.as_bytes()).as_bytes().to_vec();

    Ok((key_store_json, opaque_blob))
}

/// Derive OPAQUE login proof from passphrase + stored key store.
/// Returns the proof blob to send to server.
pub fn derive_login_proof(passphrase: &str, encrypted_key_store: &[u8]) -> Result<Vec<u8>> {
    let key_store: EncryptedKeyStore = serde_json::from_slice(encrypted_key_store)?;
    let salt = hex::decode(&key_store.kdf_salt)
        .map_err(|e| AppError::Auth(format!("Invalid salt hex: {e}")))?;
    let derived_key = derive_key(passphrase.as_bytes(), &salt)?;

    // Same as registration: hash(derived_key) as OPAQUE placeholder
    Ok(blake3::hash(derived_key.as_bytes()).as_bytes().to_vec())
}

/// Decrypt the key store after login and return the master key.
pub fn unlock_key_store(passphrase: &str, encrypted_key_store: &[u8]) -> Result<SensitiveBytes32> {
    let key_store: EncryptedKeyStore = serde_json::from_slice(encrypted_key_store)?;
    let salt = hex::decode(&key_store.kdf_salt)
        .map_err(|e| AppError::Auth(format!("Invalid salt hex: {e}")))?;
    let derived_key = derive_key(passphrase.as_bytes(), &salt)?;

    let mk_nonce_bytes = hex::decode(&key_store.master_key_nonce)
        .map_err(|e| AppError::Auth(format!("Invalid nonce hex: {e}")))?;
    let mk_ciphertext = hex::decode(&key_store.encrypted_master_key)
        .map_err(|e| AppError::Auth(format!("Invalid ciphertext hex: {e}")))?;

    let mut nonce = [0u8; 24];
    nonce.copy_from_slice(&mk_nonce_bytes);

    let master_key_bytes = decrypt(&derived_key, &nonce, &mk_ciphertext, b"master-key")?;
    SensitiveBytes32::from_slice(&master_key_bytes)
        .ok_or_else(|| AppError::Auth("Master key wrong length".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let k1 = derive_key(b"passphrase", &salt).unwrap();
        let k2 = derive_key(b"passphrase", &salt).unwrap();
        assert_eq!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn kdf_different_passphrase() {
        let salt = [42u8; 32];
        let k1 = derive_key(b"passphrase1", &salt).unwrap();
        let k2 = derive_key(b"passphrase2", &salt).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn key_store_roundtrip() {
        let passphrase = "test-passphrase-12345";
        let (key_store_json, _opaque_blob) = generate_key_store(passphrase).unwrap();

        // Should be valid JSON
        let _ks: EncryptedKeyStore = serde_json::from_slice(&key_store_json).unwrap();

        // Should be able to unlock
        let master_key = unlock_key_store(passphrase, &key_store_json).unwrap();
        assert_eq!(master_key.as_bytes().len(), 32);
    }

    #[test]
    fn key_store_wrong_passphrase() {
        let (key_store_json, _) = generate_key_store("correct-passphrase").unwrap();
        let result = unlock_key_store("wrong-passphrase", &key_store_json);
        assert!(result.is_err());
    }

    #[test]
    fn login_proof_matches_registration() {
        let passphrase = "my-secure-passphrase";
        let (key_store_json, opaque_blob) = generate_key_store(passphrase).unwrap();
        let login_proof = derive_login_proof(passphrase, &key_store_json).unwrap();
        assert_eq!(opaque_blob, login_proof);
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
}
