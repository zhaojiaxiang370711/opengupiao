use axum::{routing::get, Router};
use sqlx::PgPool;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api::{market, portfolio};

pub fn build(pool: PgPool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1/market", market::routes())
        .nest("/api/v1/portfolio", portfolio::routes(pool))
        .route("/health", get(|| async { "ok" }))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}
