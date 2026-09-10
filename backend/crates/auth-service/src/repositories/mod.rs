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
