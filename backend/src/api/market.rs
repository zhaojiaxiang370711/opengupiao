use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::openbb_bridge::OpenbbBridge;

#[derive(Serialize)]
pub struct QuoteResponse {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub interval: Option<String>,
    pub range: Option<String>,
    pub extended_hours: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryBarResponse {
    pub symbol: String,
    pub interval: String,
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub session: String,
    pub source: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/quote/:symbol", get(get_quote))
        .route("/quotes/:symbols", get(get_quotes))
        .route("/history/:symbol", get(get_history))
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

async fn get_history(
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<HistoryBarResponse>>, crate::error::AppError> {
    let interval = normalize_interval(query.interval.as_deref())?;
    let range = normalize_range(query.range.as_deref())?;
    let bridge = OpenbbBridge::get();
    let data = bridge
        .get_history(
            &symbol,
            &interval,
            &range,
            query.extended_hours.unwrap_or(true),
        )
        .await?;
    Ok(Json(data))
}

async fn search_symbols(
    Path(query): Path<String>,
) -> Result<Json<Vec<String>>, crate::error::AppError> {
    let bridge = OpenbbBridge::get();
    let results = bridge.search(&query).await?;
    Ok(Json(results))
}

fn normalize_interval(input: Option<&str>) -> Result<String, crate::error::AppError> {
    let interval = input.unwrap_or("1d").trim();
    let normalized = match interval {
        "1m" | "2m" | "5m" | "15m" | "30m" | "60m" | "90m" | "1h" | "1d" | "5d" => interval,
        "1wk" | "1W" => "1wk",
        "1mo" | "1M" => "1mo",
        _ => {
            return Err(crate::error::AppError::BadRequest(format!(
                "unsupported history interval: {interval}"
            )));
        }
    };
    Ok(normalized.to_string())
}

fn normalize_range(input: Option<&str>) -> Result<String, crate::error::AppError> {
    let range = input.unwrap_or("1mo").trim();
    let normalized = match range {
        "1d" | "5d" | "1mo" | "3mo" | "6mo" | "1y" | "2y" | "5y" | "10y" | "ytd" | "max" => range,
        _ => {
            return Err(crate::error::AppError::BadRequest(format!(
                "unsupported history range: {range}"
            )));
        }
    };
    Ok(normalized.to_string())
}
