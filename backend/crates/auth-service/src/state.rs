use common::AuthConfig;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: AuthConfig,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub dummy_password_hash: String,
}
