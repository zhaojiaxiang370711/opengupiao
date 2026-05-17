use axum::{extract::Path, routing::get, Json, Router};
use serde::Serialize;

use crate::openbb_bridge::OpenbbBridge;

#[derive(Serialize)]
pub struct QuoteResponse {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub timestamp: i64,
}

pub fn routes() -> Router {
    Router::new()
        .route("/quote/:symbol", get(get_quote))
        .route("/quotes/:symbols", get(get_quotes))
        .route("/search/:query", get(search_symbols))
}

async fn get_quote(
    Path(symbol): Path<String>,
) -> Result<Json<QuoteResponse>, crate::error::AppError> {
    let bridge = OpenbbBridge::get();
    let data = bridge.get_quote(&symbol).await?;
    Ok(Json(data))
}

async fn get_quotes(
    Path(symbols): Path<String>,
) -> Result<Json<Vec<QuoteResponse>>, crate::error::AppError> {
    let symbols = symbols
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(|symbol| symbol.to_uppercase())
        .collect::<Vec<_>>();
    let bridge = OpenbbBridge::get();
    let data = bridge.get_quotes(&symbols).await?;
    Ok(Json(data))
}

async fn search_symbols(
    Path(query): Path<String>,
) -> Result<Json<Vec<String>>, crate::error::AppError> {
    let bridge = OpenbbBridge::get();
    let results = bridge.search(&query).await?;
    Ok(Json(results))
}
