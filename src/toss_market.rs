use crate::{
    collectors::CollectionReport,
    config::Config,
    db::{Db, NewObservation},
};
use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    error::Error,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const API_BASE: &str = "https://openapi.tossinvest.com";

#[derive(Clone)]
struct CachedToken {
    value: String,
    usable_until: Instant,
}

static TOKEN_CACHE: OnceLock<Mutex<Option<CachedToken>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketIndicatorPrice {
    symbol: String,
    timestamp: Option<String>,
    last_price: String,
}

#[derive(Debug, Deserialize)]
struct MarketIndicatorResponse {
    result: Vec<MarketIndicatorPrice>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: Option<ApiErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug)]
enum FetchError {
    Unauthorized,
    Other(String),
}

fn http_client(config: &Config) -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(config.http_timeout_secs.min(20)))
        .user_agent(concat!("ECONOMICS-Radar/", env!("CARGO_PKG_VERSION")))
        .build()?)
}

fn token_cache() -> &'static Mutex<Option<CachedToken>> {
    TOKEN_CACHE.get_or_init(|| Mutex::new(None))
}

fn cached_token() -> Option<String> {
    token_cache().lock().ok().and_then(|guard| {
        guard
            .as_ref()
            .filter(|token| Instant::now() < token.usable_until)
            .map(|token| token.value.clone())
    })
}

fn invalidate_token() {
    if let Ok(mut guard) = token_cache().lock() {
        *guard = None;
    }
}

fn concise_error(status: StatusCode, value: Option<Value>) -> String {
    let detail = value
        .and_then(|value| serde_json::from_value::<ApiErrorBody>(value).ok())
        .and_then(|body| body.error);
    let code = detail.as_ref().and_then(|detail| detail.code.as_deref());
    let message = detail.as_ref().and_then(|detail| detail.message.as_deref());
    match (code, message) {
        (Some(code), Some(message)) => format!("HTTP {status} {code}: {message}"),
        (Some(code), None) => format!("HTTP {status} {code}"),
        _ if status == StatusCode::FORBIDDEN => {
            format!("HTTP {status}: this laptop's public IP is not registered in Toss Open API")
        }
        _ => format!("HTTP {status}"),
    }
}

fn issue_token(http: &Client, client_id: &str, client_secret: &str) -> Result<String, String> {
    if let Some(token) = cached_token() {
        return Ok(token);
    }
    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}",
        urlencoding::encode(client_id),
        urlencoding::encode(client_secret)
    );
    let response = http
        .post(format!("{API_BASE}/oauth2/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .map_err(|error| format!("token request: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        let value = response.json::<Value>().ok();
        return Err(format!("token: {}", concise_error(status, value)));
    }
    let token = response
        .json::<TokenResponse>()
        .map_err(|error| format!("token: invalid response: {error}"))?;
    if token.access_token.trim().is_empty() {
        return Err("token: empty access token".into());
    }
    let usable_seconds = token.expires_in.saturating_sub(60).max(1);
    if let Ok(mut guard) = token_cache().lock() {
        *guard = Some(CachedToken {
            value: token.access_token.clone(),
            usable_until: Instant::now() + Duration::from_secs(usable_seconds),
        });
    }
    Ok(token.access_token)
}

fn fetch_prices(http: &Client, token: &str) -> Result<Vec<MarketIndicatorPrice>, FetchError> {
    let response = http
        .get(format!("{API_BASE}/api/v1/market-indicators/prices"))
        .bearer_auth(token)
        .query(&[("symbols", "KOSPI,KOSDAQ")])
        .send()
        .map_err(|error| FetchError::Other(format!("request: {error}")))?;
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(FetchError::Unauthorized);
    }
    if !status.is_success() {
        let value = response.json::<Value>().ok();
        return Err(FetchError::Other(concise_error(status, value)));
    }
    response
        .json::<MarketIndicatorResponse>()
        .map(|response| response.result)
        .map_err(|error| FetchError::Other(format!("invalid response: {error}")))
}

fn authenticated_prices(
    http: &Client,
    client_id: &str,
    client_secret: &str,
) -> Result<Vec<MarketIndicatorPrice>, String> {
    let token = issue_token(http, client_id, client_secret)?;
    match fetch_prices(http, &token) {
        Ok(prices) => Ok(prices),
        Err(FetchError::Unauthorized) => {
            invalidate_token();
            let token = issue_token(http, client_id, client_secret)?;
            fetch_prices(http, &token).map_err(|error| match error {
                FetchError::Unauthorized => "prices: HTTP 401 after token renewal".into(),
                FetchError::Other(message) => format!("prices: {message}"),
            })
        }
        Err(FetchError::Other(message)) => Err(format!("prices: {message}")),
    }
}

fn normalized_timestamp(raw: &str) -> Result<String, String> {
    let timestamp =
        DateTime::parse_from_rfc3339(raw).map_err(|_| "timestamp is not RFC3339".to_string())?;
    if timestamp.with_timezone(&Utc) > Utc::now() + chrono::Duration::minutes(5) {
        return Err("timestamp is unexpectedly in the future".into());
    }
    Ok(timestamp.to_rfc3339_opts(SecondsFormat::Secs, false))
}

fn store_price(db: &Db, report: &mut CollectionReport, price: &MarketIndicatorPrice, series: &str) {
    report.attempted += 1;
    let result = (|| -> Result<bool, String> {
        let timestamp = price
            .timestamp
            .as_deref()
            .ok_or_else(|| "source timestamp is missing".to_string())
            .and_then(normalized_timestamp)?;
        let value = price
            .last_price
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| "lastPrice is not numeric".to_string())?;
        if !value.is_finite() || value <= 0.0 {
            return Err("lastPrice must be positive and finite".into());
        }
        db.put_live_quote(&NewObservation {
            source: "tossinvest".into(),
            series: series.into(),
            entity: price.symbol.clone(),
            observed_at: timestamp.clone(),
            value,
            released_at: None,
            source_asof: Some(timestamp),
            revision_id: None,
            metadata: json!({
                "provider":"Toss Securities Open API",
                "endpoint":"/api/v1/market-indicators/prices",
                "official":true,
                "symbol":price.symbol
            }),
        })
        .map_err(|error| format!("database: {error}"))
    })();
    match result {
        Ok(true) => report.stored += 1,
        Ok(false) => report.unchanged += 1,
        Err(error) => report
            .errors
            .push(format!("toss {}: {error}", price.symbol)),
    }
}

pub fn collect_indices(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let mut report = CollectionReport::default();
    let (Some(client_id), Some(client_secret)) = (
        config.tossinvest_client_id.as_deref(),
        config.tossinvest_client_secret.as_deref(),
    ) else {
        report.errors.push(
            if config.tossinvest_client_id.is_some() || config.tossinvest_client_secret.is_some() {
                "toss credentials: both client id and client secret are required"
            } else {
                "toss credentials: TOSSINVEST_CLIENT_ID and TOSSINVEST_CLIENT_SECRET are not configured"
            }
            .into(),
        );
        return Ok(report);
    };
    let http = http_client(config)?;
    let prices = match authenticated_prices(&http, client_id, client_secret) {
        Ok(prices) => prices,
        Err(error) => {
            report
                .errors
                .push(format!("toss market indicators: {error}"));
            return Ok(report);
        }
    };
    for (symbol, series) in [("KOSPI", "KOSPI_INDEX"), ("KOSDAQ", "KOSDAQ_INDEX")] {
        if let Some(price) = prices.iter().find(|price| price.symbol == symbol) {
            store_price(db, &mut report, price, series);
        } else {
            report.attempted += 1;
            report
                .errors
                .push(format!("toss {symbol}: response omitted requested symbol"));
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_market_indicator_example_parses() {
        let response: MarketIndicatorResponse = serde_json::from_value(json!({
            "result":[
                {"symbol":"KOSPI","timestamp":"2026-06-11T15:30:00+09:00","lastPrice":"2812.45"},
                {"symbol":"KOSDAQ","timestamp":"2026-06-11T15:30:00+09:00","lastPrice":"845.32"}
            ]
        }))
        .unwrap();
        assert_eq!(response.result.len(), 2);
        assert_eq!(response.result[0].last_price, "2812.45");
    }

    #[test]
    fn source_timestamp_preserves_korea_offset() {
        assert_eq!(
            normalized_timestamp("2026-06-11T15:30:00+09:00").unwrap(),
            "2026-06-11T15:30:00+09:00"
        );
    }
}
