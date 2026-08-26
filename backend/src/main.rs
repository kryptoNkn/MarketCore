use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marketcore=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/health", get(health_check));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("MarketCore server started on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "Ok"
}