mod app;
mod config;
mod db;
mod error;
mod state;

use std::net::SocketAddr;

use app::create_router;
use config::Config;
use db::{check_connection ,create_pool};
use error::AppError;
use sqlx::postgres::PgSeverity::Error;
use state::AppState;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = Config::from_env()?;

    let address: SocketAddr = config
        .server_address()
        .parse()
        .map_err(|_| AppError::InvalidServerAddress)?;

    let db = create_pool(&config.database_url)
        .await
        .map_err(AppError::Database)?;

    check_connection(&db)
        .await
        .map_err(AppError::Database)?;
    info!("Database connection established");

    let state = AppState { db };
    let app = create_router(state);

    let listener = TcpListener::bind(address)
        .await
        .map_err(AppError::ServerBind)?;

    info!("MarketCore server listening on {}", address);

    axum::serve(listener, app)
        .await
        .map_err(AppError::Server)?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marketcore=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}