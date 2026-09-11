use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, Validation, decode, encode};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{error::AppError, repositories, state::AppState};

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn register(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<TokenPair, AppError> {
    let password_hash = hash_password(password.to_owned()).await?;
    let user = repositories::create_user(&state.db, email, &password_hash).await?;
    issue_tokens(state, user.id, &user.role).await
}

pub async fn login(state: &AppState, email: &str, password: &str) -> Result<TokenPair, AppError> {
    let user = repositories::find_user_by_email(&state.db, email).await?;
    let password = password.to_owned();
    let (hash, user) = match user {
        Some(user) => (user.password_hash.clone(), Some(user)),
        None => (state.dummy_password_hash.clone(), None),
    };

    let verified = tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|_| AppError::Internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::InvalidCredentials)
    })
    .await
    .map_err(|_| AppError::Internal)?;

    match (verified, user) {
        (Ok(()), Some(user)) => issue_tokens(state, user.id, &user.role).await,
        _ => Err(AppError::InvalidCredentials),
    }
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::Internal)
    })
    .await
    .map_err(|_| AppError::Internal)?
}

fn create_token(state: &AppState, user_id: Uuid, role: &str) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Internal)?
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_owned(),
        iat: now,
        exp: now + state.auth.jwt_ttl_seconds,
    };
    encode(&Header::default(), &claims, &state.encoding_key).map_err(|_| AppError::Internal)
}

pub fn user_id_from_token(state: &AppState, token: &str) -> Result<Uuid, AppError> {
    let token_data = decode::<Claims>(token, &state.decoding_key, &Validation::default())
        .map_err(|_| AppError::Unauthorized)?;
    Uuid::parse_str(&token_data.claims.sub).map_err(|_| AppError::Unauthorized)
}

pub async fn me(state: &AppState, user_id: Uuid) -> Result<repositories::UserProfile, AppError> {
    repositories::find_user_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)
}

async fn issue_tokens(
    state: &AppState,
    user_id: Uuid,
    role: &str,
) -> Result<TokenPair, AppError> {
    let access_token = create_token(state, user_id, role)?;
    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::seconds(state.auth.refresh_ttl_seconds as i64);
    repositories::insert_refresh_token(&state.db, user_id, &token_hash, expires_at).await?;
    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    to_hex(&bytes)
}

fn hash_refresh_token(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

pub async fn refresh(state: &AppState, refresh_token: &str) -> Result<TokenPair, AppError> {
    let token_hash = hash_refresh_token(refresh_token);
    let mut tx = state.db.begin().await?;

    let stored = repositories::lock_refresh_token(&mut *tx, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if stored.revoked_at.is_some() || stored.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let user = repositories::find_user_by_id(&mut *tx, stored.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    repositories::revoke_refresh_token(&mut *tx, stored.id).await?;

    let access_token = create_token(state, user.id, &user.role)?;
    let new_refresh_token = generate_refresh_token();
    let new_hash = hash_refresh_token(&new_refresh_token);
    let expires_at = Utc::now() + Duration::seconds(state.auth.refresh_ttl_seconds as i64);
    repositories::insert_refresh_token(&mut *tx, user.id, &new_hash, expires_at).await?;

    tx.commit().await?;

    Ok(TokenPair {
        access_token,
        refresh_token: new_refresh_token,
    })
}

pub async fn logout(state: &AppState, refresh_token: &str) -> Result<(), AppError> {
    let token_hash = hash_refresh_token(refresh_token);
    repositories::revoke_refresh_token_by_hash(&state.db, &token_hash).await?;
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String,
    role: String,
    iat: u64,
    exp: u64,
}
