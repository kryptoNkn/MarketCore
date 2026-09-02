mod app;
mod db;
mod error;
mod state;

use std::net::SocketAddr;

use app::create_router;
use common::{DatabaseConfig, ServerConfig};
use db::{check_connection, create_pool};
use error::AppError;
use state::AppState;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_tracing();

    let server_config = ServerConfig::from_env()?;
    let database_config = DatabaseConfig::from_env()?;
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

    let listener = TcpListener::bind(address)
        .await
        .map_err(AppError::ServerBind)?;
    info!(%address, "auth service listening");
    axum::serve(listener, create_router(AppState { db }))
        .await
        .map_err(AppError::Server)
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("auth_service=info")
        .init();
}
