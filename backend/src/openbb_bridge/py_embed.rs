use std::collections::HashSet;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PySequence};

use crate::api::market::QuoteResponse;
use crate::error::AppError;

fn with_python<F, R>(f: F) -> Result<R, AppError>
where
    F: for<'py> FnOnce(Python<'py>) -> Result<R, AppError>,
{
    Python::attach(f)
}

fn py_err(err: PyErr) -> AppError {
    AppError::PythonBridge(err.to_string())
}

fn result_items<'py>(result: &Bound<'py, PyAny>) -> Result<Vec<Bound<'py, PyAny>>, AppError> {
    let results = result.getattr("results").map_err(py_err)?;
    if results.is_none() {
        return Ok(Vec::new());
    }

    let sequence = results
        .cast::<PySequence>()
        .map_err(|err| AppError::PythonBridge(err.to_string()))?;
    let mut items = Vec::with_capacity(sequence.len().map_err(py_err)?);
    for idx in 0..sequence.len().map_err(py_err)? {
        items.push(sequence.get_item(idx).map_err(py_err)?);
    }
    Ok(items)
}

fn extract_string(item: &Bound<'_, PyAny>, key: &str) -> Option<String> {
    item.getattr(key)
        .ok()
        .filter(|value| !value.is_none())
        .and_then(|value| value.extract::<String>().ok())
}

fn extract_f64(item: &Bound<'_, PyAny>, keys: &[&str]) -> Option<f64> {
    for key in keys {
        let Some(value) = item.getattr(*key).ok().filter(|value| !value.is_none()) else {
            continue;
        };

        if let Ok(value) = value.extract::<f64>() {
            return Some(value);
        }
        if let Ok(value) = value.extract::<i64>() {
            return Some(value as f64);
        }
    }
    None
}

fn mapping_value<'py>(item: &Bound<'py, PyAny>, key: &str) -> Option<Bound<'py, PyAny>> {
    item.call_method1("get", (key,))
        .ok()
        .filter(|value| !value.is_none())
}

fn extract_mapping_string(item: &Bound<'_, PyAny>, key: &str) -> Option<String> {
    mapping_value(item, key).and_then(|value| value.extract::<String>().ok())
}

fn extract_mapping_i64(item: &Bound<'_, PyAny>, keys: &[&str]) -> Option<i64> {
    for key in keys {
        let Some(value) = mapping_value(item, key) else {
            continue;
        };

        if let Ok(value) = value.extract::<i64>() {
            return Some(value);
        }
        if let Ok(value) = value.extract::<f64>() {
            return Some(value as i64);
        }
    }
    None
}

fn extract_mapping_f64(item: &Bound<'_, PyAny>, keys: &[&str]) -> Option<f64> {
    for key in keys {
        let Some(value) = mapping_value(item, key) else {
            continue;
        };

        if let Ok(value) = value.extract::<f64>() {
            return Some(value);
        }
        if let Ok(value) = value.extract::<i64>() {
            return Some(value as f64);
        }
        if let Ok(value) = value.extract::<String>() {
            if let Ok(value) = value.parse::<f64>() {
                return Some(value);
            }
        }
    }
    None
}

fn quote_response(
    symbol: String,
    price: f64,
    change: f64,
    change_percent: f64,
    volume: f64,
    timestamp: i64,
) -> QuoteResponse {
    QuoteResponse {
        symbol,
        price,
        change,
        change_percent,
        volume,
        timestamp,
        session: "regular".to_string(),
        market_state: None,
        quote_source: None,
        regular_price: None,
        regular_change: None,
        regular_change_percent: None,
        regular_timestamp: None,
        pre_market_price: None,
        pre_market_change: None,
        pre_market_change_percent: None,
        pre_market_timestamp: None,
        post_market_price: None,
        post_market_change: None,
        post_market_change_percent: None,
        post_market_timestamp: None,
    }
}

pub fn call_get_quote(symbol: &str) -> Result<QuoteResponse, AppError> {
    with_python(|py| {
        match call_yfinance_quote(py, symbol) {
            Ok(quote) => return Ok(quote),
            Err(err) => tracing::warn!("yfinance quote failed for {symbol}: {err}"),
        }

        match call_stooq_quote(py, symbol) {
            Ok(Some(quote)) => return Ok(quote),
            Ok(None) => {}
            Err(err) => tracing::warn!("stooq quote failed for {symbol}: {err}"),
        }

        call_openbb_quote(py, symbol)
    })
}

pub fn call_get_quotes(symbols: &[String]) -> Result<Vec<QuoteResponse>, AppError> {
    with_python(|py| {
        let mut quotes = Vec::new();
        let mut found = HashSet::new();

        for symbol in symbols {
            let normalized = symbol.trim().to_uppercase();
            if normalized.is_empty() || found.contains(&normalized) {
                continue;
            }

            match call_yfinance_quote(py, symbol) {
                Ok(quote) => {
                    found.insert(quote.symbol.to_uppercase());
                    quotes.push(quote);
                    continue;
                }
                Err(err) => tracing::warn!("yfinance quote failed for {symbol}: {err}"),
            }

            match call_stooq_quote(py, symbol) {
                Ok(Some(quote)) => {
                    found.insert(quote.symbol.to_uppercase());
                    quotes.push(quote);
                    continue;
                }
                Ok(None) => {}
                Err(err) => tracing::warn!("stooq quote failed for {symbol}: {err}"),
            }

            if let Ok(quote) = call_openbb_quote(py, symbol) {
                found.insert(quote.symbol.to_uppercase());
                quotes.push(quote);
            }
        }

        Ok(quotes)
    })
}

fn call_stooq_quote(py: Python<'_>, symbol: &str) -> Result<Option<QuoteResponse>, AppError> {
    let symbols = vec![symbol.to_string()];
    Ok(call_stooq_quotes(py, &symbols)?.into_iter().next())
}

fn call_stooq_quotes(py: Python<'_>, symbols: &[String]) -> Result<Vec<QuoteResponse>, AppError> {
    let stooq_symbols = symbols
        .iter()
        .map(|symbol| stooq_symbol(symbol))
        .filter(|symbol| !symbol.is_empty())
        .collect::<Vec<_>>();

    if stooq_symbols.is_empty() {
        return Ok(Vec::new());
    }

    let urllib = py
        .import("urllib.request")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    let kwargs = PyDict::new(py);
    kwargs
        .set_item("timeout", 5.0)
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let url = format!(
        "https://stooq.com/q/l/?s={}&f=sd2t2ohlcvp&h&e=csv",
        stooq_symbols.join("+")
    );
    let response = urllib
        .call_method("urlopen", (url,), Some(&kwargs))
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    let body = response
        .call_method0("read")
        .and_then(|bytes| bytes.call_method1("decode", ("utf-8",)))
        .and_then(|text| text.extract::<String>())
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    let _ = response.call_method0("close");

    Ok(parse_stooq_quotes(symbols, &body))
}

fn stooq_symbol(symbol: &str) -> String {
    let symbol = symbol.trim().to_ascii_lowercase();
    if symbol.contains('.') {
        symbol
    } else {
        format!("{symbol}.us")
    }
}

fn parse_stooq_quotes(symbols: &[String], body: &str) -> Vec<QuoteResponse> {
    let mut lines = body.lines();
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers: Vec<&str> = header_line.split(',').collect();

    lines
        .enumerate()
        .filter_map(|(index, line)| {
            let values: Vec<&str> = line.split(',').collect();
            let fallback_symbol = symbols.get(index).map(String::as_str).unwrap_or_default();
            parse_stooq_row(fallback_symbol, &headers, &values)
        })
        .collect()
}

fn parse_stooq_row(symbol: &str, headers: &[&str], values: &[&str]) -> Option<QuoteResponse> {
    let field = |name: &str| {
        headers
            .iter()
            .position(|header| header.eq_ignore_ascii_case(name))
            .and_then(|idx| values.get(idx).copied())
            .filter(|value| !value.is_empty() && *value != "N/D")
    };

    let price = field("Close")?.parse::<f64>().ok()?;
    let prev_close = field("Prev").and_then(|value| value.parse::<f64>().ok());
    let change = prev_close.map(|prev| price - prev).unwrap_or(0.0);
    let change_percent = prev_close
        .filter(|prev| *prev != 0.0)
        .map(|prev| change / prev * 100.0)
        .unwrap_or(0.0);
    let volume = field("Volume")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);

    Some(quote_response(
        field("Symbol")
            .and_then(|value| value.split('.').next())
            .map(|value| value.to_uppercase())
            .unwrap_or_else(|| symbol.to_uppercase()),
        price,
        change,
        change_percent,
        volume,
        chrono::Utc::now().timestamp(),
    ))
}

fn call_yfinance_quote(py: Python<'_>, symbol: &str) -> Result<QuoteResponse, AppError> {
    let yfinance = py
        .import("yfinance")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    let ticker = yfinance
        .getattr("Ticker")
        .and_then(|ticker| ticker.call1((symbol,)))
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    let info = ticker
        .call_method0("get_info")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let now = chrono::Utc::now().timestamp();
    let symbol = extract_mapping_string(&info, "symbol").unwrap_or_else(|| symbol.to_uppercase());
    let market_state = extract_mapping_string(&info, "marketState");
    let quote_source = extract_mapping_string(&info, "quoteSourceName");

    let regular_price = extract_mapping_f64(
        &info,
        &["regularMarketPrice", "currentPrice", "postMarketPrice"],
    );
    let previous_close =
        extract_mapping_f64(&info, &["regularMarketPreviousClose", "previousClose"]);
    let regular_change = extract_mapping_f64(&info, &["regularMarketChange"]).or_else(|| {
        regular_price
            .zip(previous_close)
            .map(|(price, previous)| price - previous)
    });
    let regular_change_percent = extract_mapping_f64(&info, &["regularMarketChangePercent"])
        .or_else(|| {
            regular_change
                .zip(previous_close)
                .and_then(|(change, previous)| {
                    if previous == 0.0 {
                        None
                    } else {
                        Some(change / previous * 100.0)
                    }
                })
        });
    let regular_timestamp = extract_mapping_i64(&info, &["regularMarketTime"]);

    let pre_market_price = extract_mapping_f64(&info, &["preMarketPrice"]);
    let pre_market_change = extract_mapping_f64(&info, &["preMarketChange"]).or_else(|| {
        pre_market_price
            .zip(regular_price)
            .map(|(price, regular)| price - regular)
    });
    let pre_market_change_percent = extract_mapping_f64(&info, &["preMarketChangePercent"])
        .or_else(|| {
            pre_market_change
                .zip(regular_price)
                .and_then(|(change, regular)| {
                    if regular == 0.0 {
                        None
                    } else {
                        Some(change / regular * 100.0)
                    }
                })
        });
    let pre_market_timestamp = extract_mapping_i64(&info, &["preMarketTime"]);

    let post_market_price = extract_mapping_f64(&info, &["postMarketPrice"]);
    let post_market_change = extract_mapping_f64(&info, &["postMarketChange"]).or_else(|| {
        post_market_price
            .zip(regular_price)
            .map(|(price, regular)| price - regular)
    });
    let post_market_change_percent = extract_mapping_f64(&info, &["postMarketChangePercent"])
        .or_else(|| {
            post_market_change
                .zip(regular_price)
                .and_then(|(change, regular)| {
                    if regular == 0.0 {
                        None
                    } else {
                        Some(change / regular * 100.0)
                    }
                })
        });
    let post_market_timestamp = extract_mapping_i64(&info, &["postMarketTime"]);

    let state = market_state
        .as_deref()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let use_pre_market = state.contains("PRE") && pre_market_price.is_some();
    let use_post_market = (state.contains("POST") || state == "CLOSED")
        && post_market_price.is_some()
        && post_market_timestamp.unwrap_or(0) >= regular_timestamp.unwrap_or(0);

    let (session, price, change, change_percent, timestamp) = if use_pre_market {
        (
            "pre_market",
            pre_market_price,
            pre_market_change,
            pre_market_change_percent,
            pre_market_timestamp,
        )
    } else if use_post_market {
        (
            "post_market",
            post_market_price,
            post_market_change,
            post_market_change_percent,
            post_market_timestamp,
        )
    } else {
        (
            "regular",
            regular_price,
            regular_change,
            regular_change_percent,
            regular_timestamp,
        )
    };

    let price = price
        .or(regular_price)
        .ok_or_else(|| AppError::PythonBridge(format!("no quote price for {symbol}")))?;

    Ok(QuoteResponse {
        symbol,
        price,
        change: change.unwrap_or(0.0),
        change_percent: change_percent.unwrap_or(0.0),
        volume: extract_mapping_f64(&info, &["regularMarketVolume", "volume"]).unwrap_or(0.0),
        timestamp: timestamp.or(regular_timestamp).unwrap_or(now),
        session: session.to_string(),
        market_state,
        quote_source,
        regular_price,
        regular_change,
        regular_change_percent,
        regular_timestamp,
        pre_market_price,
        pre_market_change,
        pre_market_change_percent,
        pre_market_timestamp,
        post_market_price,
        post_market_change,
        post_market_change_percent,
        post_market_timestamp,
    })
}

fn call_openbb_quote(py: Python<'_>, symbol: &str) -> Result<QuoteResponse, AppError> {
    let openbb = py
        .import("openbb")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let obb = openbb
        .getattr("obb")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let kwargs = PyDict::new(py);
    kwargs
        .set_item("symbol", symbol)
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;
    kwargs
        .set_item("provider", "yfinance")
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let result = obb
        .getattr("equity")
        .and_then(|equity| equity.getattr("price"))
        .and_then(|price| price.call_method("quote", (), Some(&kwargs)))
        .map_err(|e| AppError::PythonBridge(e.to_string()))?;

    let Some(item) = result_items(&result)?.into_iter().next() else {
        return Ok(quote_response(
            symbol.to_uppercase(),
            0.0,
            0.0,
            0.0,
            0.0,
            chrono::Utc::now().timestamp(),
        ));
    };

    let price = extract_f64(&item, &["last_price", "close", "bid", "ask"]).unwrap_or(0.0);
    let prev_close = extract_f64(&item, &["prev_close"]);
    let change = extract_f64(&item, &["change"])
        .or_else(|| prev_close.map(|prev_close| price - prev_close))
        .unwrap_or(0.0);
    let change_percent = extract_f64(&item, &["change_percent"])
        .or_else(|| {
            prev_close
                .filter(|prev_close| *prev_close != 0.0)
                .map(|prev_close| change / prev_close * 100.0)
        })
        .unwrap_or(0.0);

    Ok(quote_response(
        extract_string(&item, "symbol").unwrap_or_else(|| symbol.to_uppercase()),
        price,
        change,
        change_percent,
        extract_f64(&item, &["volume", "exchange_volume"]).unwrap_or(0.0),
        chrono::Utc::now().timestamp(),
    ))
}

pub fn call_search(query: &str) -> Result<Vec<String>, AppError> {
    with_python(|py| {
        let openbb = py
            .import("openbb")
            .map_err(|e| AppError::PythonBridge(e.to_string()))?;

        let obb = openbb
            .getattr("obb")
            .map_err(|e| AppError::PythonBridge(e.to_string()))?;

        let kwargs = PyDict::new(py);
        kwargs
            .set_item("query", query)
            .map_err(|e| AppError::PythonBridge(e.to_string()))?;
        kwargs
            .set_item("provider", "sec")
            .map_err(|e| AppError::PythonBridge(e.to_string()))?;

        let result = obb
            .getattr("equity")
            .and_then(|equity| equity.call_method("search", (), Some(&kwargs)))
            .map_err(|e| AppError::PythonBridge(e.to_string()))?;

        let items = result_items(&result)?
            .into_iter()
            .filter_map(|item| extract_string(&item, "symbol"))
            .collect();

        Ok(items)
    })
}
