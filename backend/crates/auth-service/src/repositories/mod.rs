use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub password_hash: String,
    pub role: String,
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        "INSERT INTO users (email, password_hash)
         VALUES ($1, $2)
         RETURNING id, password_hash, role",
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        "SELECT id, password_hash, role FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

pub async fn find_user_by_id(
    executor: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<UserProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserProfile>(
        "SELECT id, email, role FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

#[derive(Debug, FromRow)]
pub struct RefreshRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn insert_refresh_token(
    executor: impl sqlx::PgExecutor<'_>,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(executor)
    .await
    .map(|_| ())
}

pub async fn lock_refresh_token(
    executor: impl sqlx::PgExecutor<'_>,
    token_hash: &str,
) -> Result<Option<RefreshRecord>, sqlx::Error> {
    sqlx::query_as::<_, RefreshRecord>(
        "SELECT id, user_id, expires_at, revoked_at
         FROM refresh_tokens
         WHERE token_hash = $1
         FOR UPDATE",
    )
    .bind(token_hash)
    .fetch_optional(executor)
    .await
}

pub async fn revoke_refresh_token(
    executor: impl sqlx::PgExecutor<'_>,
    token_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE refresh_tokens
         SET revoked_at = NOW()
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(token_id)
    .execute(executor)
    .await
    .map(|_| ())
}

pub async fn revoke_refresh_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE refresh_tokens
         SET revoked_at = NOW()
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(pool)
    .await
    .map(|_| ())
}
