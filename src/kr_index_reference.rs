use chrono::Utc;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, REFERER},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const PROVIDER_API_ROOT: &str = "https://m.stock.naver.com/api/index";
const PROVIDER_REFERER: &str = "https://m.stock.naver.com/";

#[derive(Debug, Deserialize)]
struct ProviderIndexQuote {
    #[serde(rename = "stockName")]
    stock_name: String,
    #[serde(rename = "closePrice")]
    close_price: String,
    #[serde(rename = "compareToPreviousClosePrice")]
    change: String,
    #[serde(rename = "fluctuationsRatio")]
    change_pct: String,
    #[serde(rename = "marketStatus")]
    market_status: String,
    #[serde(rename = "localTradedAt")]
    observed_at: String,
    #[serde(rename = "delayTimeName", default)]
    delay_label: Option<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct IndexReferenceQuote {
    pub code: String,
    pub name: String,
    pub value: f64,
    pub change: f64,
    pub change_pct: f64,
    pub market_status: String,
    pub market_status_label: String,
    pub observed_at: String,
    pub delay_label: String,
    pub source: &'static str,
    pub source_url: String,
    pub used_in_risk_engine: bool,
}

#[derive(Debug, Serialize)]
pub struct IndexReferenceResponse {
    pub status: &'static str,
    pub fetched_at: String,
    pub refresh_seconds: u16,
    pub quotes: Vec<IndexReferenceQuote>,
    pub errors: Vec<String>,
}

fn parse_number(value: &str) -> Result<f64, String> {
    value
        .replace(',', "")
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid numeric value: {value}"))
}

fn market_status_label(status: &str) -> &'static str {
    match status {
        "OPEN" => "장중",
        "CLOSE" => "장마감",
        "PRE_OPEN" => "장전",
        "AFTER_HOURS" => "시간외",
        _ => "상태 확인 필요",
    }
}

fn normalize_quote(code: &str, payload: ProviderIndexQuote) -> Result<IndexReferenceQuote, String> {
    Ok(IndexReferenceQuote {
        code: code.to_string(),
        name: payload.stock_name,
        value: parse_number(&payload.close_price)?,
        change: parse_number(&payload.change)?,
        change_pct: parse_number(&payload.change_pct)?,
        market_status_label: market_status_label(&payload.market_status).to_string(),
        market_status: payload.market_status,
        observed_at: payload.observed_at,
        delay_label: payload
            .delay_label
            .unwrap_or_else(|| "지연 여부 미표시".into()),
        source: "Npay 증권",
        source_url: format!("https://m.stock.naver.com/domestic/index/{code}"),
        used_in_risk_engine: false,
    })
}

pub fn fetch() -> IndexReferenceResponse {
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static(PROVIDER_REFERER));
    let client = Client::builder()
        .default_headers(headers)
        .user_agent(format!(
            "EconomicsRadar/{} (+local personal market dashboard)",
            env!("CARGO_PKG_VERSION")
        ))
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(6))
        .build();

    let mut quotes = Vec::new();
    let mut errors = Vec::new();
    match client {
        Ok(client) => {
            for code in ["KOSPI", "KOSDAQ"] {
                let result = client
                    .get(format!("{PROVIDER_API_ROOT}/{code}/basic"))
                    .send()
                    .and_then(reqwest::blocking::Response::error_for_status)
                    .and_then(reqwest::blocking::Response::json::<ProviderIndexQuote>)
                    .map_err(|error| error.to_string())
                    .and_then(|payload| normalize_quote(code, payload));
                match result {
                    Ok(quote) => quotes.push(quote),
                    Err(error) => errors.push(format!("{code}: {error}")),
                }
            }
        }
        Err(error) => errors.push(format!("HTTP client: {error}")),
    }

    let status = match quotes.len() {
        2 => "ok",
        1 => "partial",
        _ => "unavailable",
    };
    IndexReferenceResponse {
        status,
        fetched_at: Utc::now().to_rfc3339(),
        refresh_seconds: 30,
        quotes,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_payload_is_normalized_without_rounding_original_values() {
        let payload: ProviderIndexQuote = serde_json::from_str(
            r#"{
                "stockName":"코스피",
                "closePrice":"6,696.96",
                "compareToPreviousClosePrice":"-215.99",
                "fluctuationsRatio":"-3.12",
                "marketStatus":"CLOSE",
                "localTradedAt":"2026-08-24T18:59:00+09:00",
                "delayTimeName":"실시간"
            }"#,
        )
        .unwrap();
        let quote = normalize_quote("KOSPI", payload).unwrap();
        assert_eq!(quote.value, 6_696.96);
        assert_eq!(quote.change, -215.99);
        assert_eq!(quote.change_pct, -3.12);
        assert_eq!(quote.market_status_label, "장마감");
        assert_eq!(quote.delay_label, "실시간");
        assert!(!quote.used_in_risk_engine);
    }

    #[test]
    fn malformed_numeric_value_is_rejected_instead_of_filled() {
        assert!(parse_number("—").is_err());
    }
}
