use axum::{extract::State, routing::get, Json, Router};
use sqlx::PgPool;

use crate::db::models::{PortfolioHolding, WatchlistItem};
use crate::error::AppError;

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/watchlist", get(get_watchlist))
        .route("/holdings", get(get_holdings))
        .with_state(pool)
}

async fn get_watchlist(State(pool): State<PgPool>) -> Result<Json<Vec<WatchlistItem>>, AppError> {
    let items =
        sqlx::query_as::<_, WatchlistItem>("SELECT id, user_id, symbol, added_at FROM watchlist")
            .fetch_all(&pool)
            .await?;
    Ok(Json(items))
}

async fn get_holdings(State(pool): State<PgPool>) -> Result<Json<Vec<PortfolioHolding>>, AppError> {
    let items = sqlx::query_as::<_, PortfolioHolding>(
        "SELECT id, user_id, symbol, quantity, avg_cost, updated_at
         FROM portfolio_holdings
         ORDER BY updated_at DESC",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(items))
}
