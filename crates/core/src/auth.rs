use uuid::Uuid;

use crate::api::{ApiClient, LoginRequest, RegisterRequest};
use crate::crypto::{self, SensitiveBytes32};
use crate::Result;

/// Authenticated session holding user identity, JWT, and decrypted master key.
#[derive(Clone)]
pub struct Session {
    pub user_id: Uuid,
    pub token: String,
    master_key: SensitiveBytes32,
}

impl Session {
    pub fn master_key(&self) -> &SensitiveBytes32 {
        &self.master_key
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("user_id", &self.user_id)
            .field("token", &"[redacted]")
            .field("master_key", &"[redacted]")
            .finish()
    }
}

/// Register a new user.
///
/// 1. Generate all key material client-side (Argon2id → MK → KEM/X25519/DSA/Ed25519)
/// 2. Encrypt key store with passphrase-derived key
/// 3. Send encrypted key store + OPAQUE blob to server
/// 4. Server stores opaque blobs, issues JWT
pub async fn register(client: &ApiClient, passphrase: &str) -> Result<Session> {
    // Generate encrypted key store and OPAQUE registration blob
    let (key_store_json, opaque_blob) = crypto::generate_key_store(passphrase)?;

    // Send to server
    let resp = client
        .register(&RegisterRequest {
            opaque_registration: opaque_blob,
            encrypted_key_store: key_store_json.clone(),
        })
        .await?;

    // Decrypt master key locally
    let master_key = crypto::unlock_key_store(passphrase, &key_store_json)?;

    Ok(Session {
        user_id: resp.user_id,
        token: resp.token,
        master_key,
    })
}

/// Login an existing user.
///
/// 1. Derive OPAQUE login proof from passphrase + encrypted key store
/// 2. Server verifies proof, returns JWT + encrypted key store
/// 3. Client decrypts key store locally to recover master key
pub async fn login(client: &ApiClient, user_id: Uuid, passphrase: &str) -> Result<Session> {
    // We need the key store to derive the login proof, but we don't have it yet.
    // For the current simplified OPAQUE, we need to do a pre-fetch or use a
    // two-step flow. For now, derive proof from passphrase directly.
    //
    // In full OPAQUE this becomes a multi-round protocol.

    // Derive a deterministic proof from just the passphrase + user_id as salt
    let salt = blake3::hash(user_id.as_bytes()).as_bytes().to_vec();
    let derived = crypto::derive_key(passphrase.as_bytes(), &salt)?;
    let proof = blake3::hash(derived.as_bytes()).as_bytes().to_vec();

    let resp = client
        .login(&LoginRequest {
            user_id,
            opaque_login_proof: proof,
        })
        .await?;

    // Decrypt master key from the returned key store
    let master_key = crypto::unlock_key_store(passphrase, &resp.encrypted_key_store)?;

    Ok(Session {
        user_id,
        token: resp.token,
        master_key,
    })
}
