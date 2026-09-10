mod app;
mod db;
mod error;
mod handlers;
mod models;
mod repositories;
mod services;
mod state;

use std::net::SocketAddr;

use app::create_router;
use common::{AuthConfig, DatabaseConfig, ServerConfig};
use db::{check_connection, create_pool};
use error::AppError;
use jsonwebtoken::EncodingKey;
use state::AppState;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_tracing();

    let server_config = ServerConfig::from_env()?;
    let database_config = DatabaseConfig::from_env()?;
    let auth_config = AuthConfig::from_env()?;
    let address: SocketAddr = server_config
        .server_address()
        .parse()
        .map_err(|_| AppError::InvalidServerAddress)?;
    let db = create_pool(&database_config.database_url)
        .await
        .map_err(AppError::Database)?;
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .map_err(AppError::Migrations)?;
    check_connection(&db).await.map_err(AppError::Database)?;

    let encoding_key = EncodingKey::from_secret(auth_config.jwt_secret.as_bytes());
    let dummy_password_hash = services::hash_password("timing-dummy".to_owned()).await?;

    let listener = TcpListener::bind(address)
        .await
        .map_err(AppError::ServerBind)?;
    info!(%address, "auth service listening");
    axum::serve(
        listener,
        create_router(AppState {
            db,
            auth: auth_config,
            encoding_key,
            dummy_password_hash,
        }),
    )
    .await
    .map_err(AppError::Server)
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=info,tower_http=info".into()),
        )
        .init();
}
