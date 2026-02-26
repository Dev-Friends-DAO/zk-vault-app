use uuid::Uuid;

use crate::api::{ApiClient, LoginRequest, RegisterRequest};
use crate::Result;

/// Authenticated session holding user identity and JWT.
#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: Uuid,
    pub token: String,
    pub encrypted_key_store: Vec<u8>,
}

/// Register a new user.
///
/// In a full implementation, `opaque_registration` and `encrypted_key_store`
/// are derived client-side from the user's passphrase using Argon2id + OPAQUE.
/// For now this accepts pre-built blobs.
pub async fn register(
    client: &ApiClient,
    opaque_registration: Vec<u8>,
    encrypted_key_store: Vec<u8>,
) -> Result<Session> {
    let resp = client
        .register(&RegisterRequest {
            opaque_registration,
            encrypted_key_store: encrypted_key_store.clone(),
        })
        .await?;

    Ok(Session {
        user_id: resp.user_id,
        token: resp.token,
        encrypted_key_store,
    })
}

/// Login an existing user.
///
/// Returns the session with the encrypted key store that the client
/// must decrypt locally using the passphrase.
pub async fn login(
    client: &ApiClient,
    user_id: Uuid,
    opaque_login_proof: Vec<u8>,
) -> Result<Session> {
    let resp = client
        .login(&LoginRequest {
            user_id,
            opaque_login_proof,
        })
        .await?;

    Ok(Session {
        user_id,
        token: resp.token,
        encrypted_key_store: resp.encrypted_key_store,
    })
}
