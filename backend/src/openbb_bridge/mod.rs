mod py_embed;

use std::sync::OnceLock;
use tokio::sync::Mutex;

use crate::api::market::QuoteResponse;
use crate::error::AppError;

static INSTANCE: OnceLock<OpenbbBridge> = OnceLock::new();

pub struct OpenbbBridge {
    inner: Mutex<()>,
}

impl OpenbbBridge {
    pub fn get() -> &'static Self {
        INSTANCE.get_or_init(|| Self {
            inner: Mutex::new(()),
        })
    }

    pub async fn get_quote(&self, symbol: &str) -> Result<QuoteResponse, AppError> {
        let _guard = self.inner.lock().await;
        py_embed::call_get_quote(symbol)
    }

    pub async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<QuoteResponse>, AppError> {
        let _guard = self.inner.lock().await;
        py_embed::call_get_quotes(symbols)
    }

    pub async fn search(&self, query: &str) -> Result<Vec<String>, AppError> {
        let _guard = self.inner.lock().await;
        py_embed::call_search(query)
    }
}
