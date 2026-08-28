mod app;
mod config;
mod error;

use std::net::SocketAddr;

use app::create_router;
use config::Config;
use error::AppError;
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

    let app = create_router();

    let listener = TcpListener::bind(address)
        .await
        .map_err(AppError::ServerBind)?;

    info!("MarketCore server listening on {}", address);

    axum::serve(listener, app).await.map_err(AppError::Server)?;

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
