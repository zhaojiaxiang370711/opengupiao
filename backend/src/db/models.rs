use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kline {
    pub id: Uuid,
    pub symbol: String,
    pub interval: String,
    pub open_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchlistItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioHolding {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub updated_at: DateTime<Utc>,
}
