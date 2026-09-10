use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use jsonwebtoken::{Header, encode};
use rand_core::OsRng;
use uuid::Uuid;

use crate::{error::AppError, repositories, state::AppState};

pub async fn register(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let password_hash = hash_password(password.to_owned()).await?;
    let user = repositories::create_user(&state.db, email, &password_hash).await?;
    create_token(state, user.id, &user.role)
}

pub async fn login(state: &AppState, email: &str, password: &str) -> Result<String, AppError> {
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
        (Ok(()), Some(user)) => create_token(state, user.id, &user.role),
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

#[derive(Debug, serde::Serialize)]
struct Claims {
    sub: String,
    role: String,
    iat: u64,
    exp: u64,
}
