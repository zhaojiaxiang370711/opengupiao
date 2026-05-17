use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Python bridge error: {0}")]
    PythonBridge(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::PythonBridge(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e.clone()),
            Self::BadRequest(e) => (StatusCode::BAD_REQUEST, e.clone()),
            Self::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, Json(json!({ "error": msg }))).into_response()
    }
}
