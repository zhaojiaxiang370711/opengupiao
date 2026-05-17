mod api;
mod config;
mod db;
mod error;
mod openbb_bridge;
mod services;

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let cfg = config::Config::from_env();

    tracing::info!("Connecting to database...");
    let pool = db::init_pool(&cfg.database_url).await?;
    db::run_migrations(&pool).await?;

    tracing::info!("Initializing OpenBB bridge...");
    openbb_bridge::OpenbbBridge::get();

    let app = api::router::build(pool);

    let addr: SocketAddr = cfg.bind_addr.parse()?;
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
