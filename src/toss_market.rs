use crate::{
    collectors::CollectionReport,
    config::Config,
    db::{Db, NewObservation},
};
use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::{blocking::Client, header::RETRY_AFTER, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const API_BASE: &str = "https://openapi.tossinvest.com";
const MARKET_SYMBOLS: &str =
    "KOSPI,KOSDAQ,KR_BOND_2Y,KR_BOND_3Y,KR_BOND_5Y,KR_BOND_10Y,KR_BOND_20Y,KR_BOND_30Y";

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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangeRate {
    rate: String,
    mid_rate: String,
    basis_point: String,
    valid_from: String,
    valid_until: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradingAmount {
    buy_amount: String,
    sell_amount: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstitutionBreakdown {
    financial_investment: TradingAmount,
    insurance: TradingAmount,
    trust: TradingAmount,
    private_equity_fund: TradingAmount,
    bank: TradingAmount,
    other_financial_institution: TradingAmount,
    pension_fund: TradingAmount,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstitutionTradingAmount {
    buy_amount: String,
    sell_amount: String,
    breakdown: InstitutionBreakdown,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvestorTradingRecord {
    date: String,
    updated_at: String,
    individual: TradingAmount,
    foreigner: TradingAmount,
    institution: InstitutionTradingAmount,
    other_corporation: TradingAmount,
}

#[derive(Clone, Debug, Deserialize)]
struct InvestorTradingResponse {
    records: Vec<InvestorTradingRecord>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketCandle {
    timestamp: String,
    open_price: String,
    high_price: String,
    low_price: String,
    close_price: String,
    volume: String,
}

#[derive(Clone, Debug, Deserialize)]
struct MarketCandlePage {
    candles: Vec<MarketCandle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RankingPrice {
    last_price: String,
    base_price: String,
    change_rate: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RankingItem {
    rank: u32,
    symbol: String,
    currency: String,
    price: RankingPrice,
    trading_volume: String,
    trading_amount: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    market: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    security_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RankingResponse {
    ranked_at: Option<String>,
    rankings: Vec<RankingItem>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StockInfo {
    symbol: String,
    name: String,
    market: String,
    security_type: String,
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
    RateLimited(u64),
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

fn fetch_json(
    http: &Client,
    token: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<Value, FetchError> {
    let response = http
        .get(format!("{API_BASE}{path}"))
        .bearer_auth(token)
        .query(query)
        .send()
        .map_err(|error| FetchError::Other(format!("request: {error}")))?;
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(FetchError::Unauthorized);
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let wait = response
            .headers()
            .get(RETRY_AFTER)
            .or_else(|| response.headers().get("X-RateLimit-Reset"))
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(1)
            .clamp(1, 3);
        return Err(FetchError::RateLimited(wait));
    }
    if !status.is_success() {
        let value = response.json::<Value>().ok();
        return Err(FetchError::Other(concise_error(status, value)));
    }
    response
        .json::<Value>()
        .map_err(|error| FetchError::Other(format!("invalid response: {error}")))
}

fn authenticated_json(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<Value, String> {
    let mut renewed = false;
    let mut rate_retried = false;
    loop {
        let token = issue_token(http, client_id, client_secret)?;
        match fetch_json(http, &token, path, query) {
            Ok(value) => return Ok(value),
            Err(FetchError::Unauthorized) if !renewed => {
                invalidate_token();
                renewed = true;
            }
            Err(FetchError::RateLimited(wait)) if !rate_retried => {
                thread::sleep(Duration::from_secs(wait));
                rate_retried = true;
            }
            Err(FetchError::Unauthorized) => {
                return Err(format!("{path}: HTTP 401 after token renewal"));
            }
            Err(FetchError::RateLimited(_)) => {
                return Err(format!("{path}: HTTP 429 after Retry-After retry"));
            }
            Err(FetchError::Other(message)) => return Err(format!("{path}: {message}")),
        }
    }
}

fn credentials(config: &Config) -> Result<(&str, &str), String> {
    match (
        config.tossinvest_client_id.as_deref(),
        config.tossinvest_client_secret.as_deref(),
    ) {
        (Some(client_id), Some(client_secret)) => Ok((client_id, client_secret)),
        _ if config.tossinvest_client_id.is_some() || config.tossinvest_client_secret.is_some() => {
            Err("both client id and client secret are required".into())
        }
        _ => Err("TOSSINVEST_CLIENT_ID and TOSSINVEST_CLIENT_SECRET are not configured".into()),
    }
}

fn result_value(value: Value, endpoint: &str) -> Result<Value, String> {
    value
        .get("result")
        .cloned()
        .ok_or_else(|| format!("{endpoint}: response has no result"))
}

fn normalized_timestamp(raw: &str) -> Result<String, String> {
    let timestamp =
        DateTime::parse_from_rfc3339(raw).map_err(|_| "timestamp is not RFC3339".to_string())?;
    if timestamp.with_timezone(&Utc) > Utc::now() + chrono::Duration::minutes(5) {
        return Err("timestamp is unexpectedly in the future".into());
    }
    Ok(timestamp.to_rfc3339_opts(SecondsFormat::Secs, false))
}

fn number(raw: &str, field: &str) -> Result<f64, String> {
    let value = raw
        .replace(',', "")
        .parse::<f64>()
        .map_err(|_| format!("{field} is not numeric"))?;
    if !value.is_finite() {
        return Err(format!("{field} must be finite"));
    }
    Ok(value)
}

fn record_result(report: &mut CollectionReport, result: Result<bool, String>, label: &str) {
    report.attempted += 1;
    match result {
        Ok(true) => report.stored += 1,
        Ok(false) => report.unchanged += 1,
        Err(error) => report.errors.push(format!("toss {label}: {error}")),
    }
}

#[allow(clippy::too_many_arguments)]
fn store_live_numeric(
    db: &Db,
    report: &mut CollectionReport,
    series: &str,
    entity: &str,
    timestamp: &str,
    value: f64,
    endpoint: &str,
    metadata: Value,
) {
    let result = (|| -> Result<bool, String> {
        let timestamp = normalized_timestamp(timestamp)?;
        db.put_live_quote(&NewObservation {
            source: "tossinvest".into(),
            series: series.into(),
            entity: entity.into(),
            observed_at: timestamp.clone(),
            value,
            released_at: None,
            source_asof: Some(timestamp),
            revision_id: None,
            metadata: json!({
                "provider":"Toss Securities Open API",
                "endpoint":endpoint,
                "official":true,
                "details":metadata
            }),
        })
        .map_err(|error| format!("database: {error}"))
    })();
    record_result(report, result, series);
}

fn store_prices(db: &Db, report: &mut CollectionReport, prices: &[MarketIndicatorPrice]) {
    let mappings = [
        ("KOSPI", "KOSPI_INDEX"),
        ("KOSDAQ", "KOSDAQ_INDEX"),
        ("KR_BOND_2Y", "KR_BOND_2Y_YIELD"),
        ("KR_BOND_3Y", "KR_BOND_3Y_YIELD"),
        ("KR_BOND_5Y", "KR_BOND_5Y_YIELD"),
        ("KR_BOND_10Y", "KR_BOND_10Y_YIELD"),
        ("KR_BOND_20Y", "KR_BOND_20Y_YIELD"),
        ("KR_BOND_30Y", "KR_BOND_30Y_YIELD"),
    ];
    for (symbol, series) in mappings {
        let Some(price) = prices.iter().find(|price| price.symbol == symbol) else {
            report.attempted += 1;
            report
                .errors
                .push(format!("toss {symbol}: response omitted requested symbol"));
            continue;
        };
        let Some(timestamp) = price.timestamp.as_deref() else {
            report.attempted += 1;
            report
                .errors
                .push(format!("toss {symbol}: source timestamp is missing"));
            continue;
        };
        match number(&price.last_price, "lastPrice") {
            Ok(value) if value > 0.0 => store_live_numeric(
                db,
                report,
                series,
                symbol,
                timestamp,
                value,
                "/api/v1/market-indicators/prices",
                json!({"symbol":symbol}),
            ),
            Ok(_) => {
                report.attempted += 1;
                report
                    .errors
                    .push(format!("toss {symbol}: lastPrice must be positive"));
            }
            Err(error) => {
                report.attempted += 1;
                report.errors.push(format!("toss {symbol}: {error}"));
            }
        }
    }
}

fn store_exchange_rate(db: &Db, report: &mut CollectionReport, rate: &ExchangeRate) {
    for (series, raw, field) in [
        ("USD_KRW_MID", rate.mid_rate.as_str(), "midRate"),
        ("USD_KRW_BUY", rate.rate.as_str(), "rate"),
        ("USD_KRW_BASIS_BP", rate.basis_point.as_str(), "basisPoint"),
    ] {
        match number(raw, field) {
            Ok(value) => store_live_numeric(
                db,
                report,
                series,
                "USD/KRW",
                &rate.valid_from,
                value,
                "/api/v1/exchange-rate",
                json!({
                    "baseCurrency":"USD",
                    "quoteCurrency":"KRW",
                    "field":field,
                    "validUntil":rate.valid_until
                }),
            ),
            Err(error) => {
                report.attempted += 1;
                report.errors.push(format!("toss exchange rate: {error}"));
            }
        }
    }
}

fn store_observation(db: &Db, report: &mut CollectionReport, observation: NewObservation) {
    let label = observation.series.clone();
    let result = db
        .put(&observation)
        .map_err(|error| format!("database: {error}"));
    record_result(report, result, &label);
}

fn amount_observation(
    market: &str,
    category: &str,
    date: &str,
    updated_at: &str,
    amount: &TradingAmount,
) -> Result<NewObservation, String> {
    let buy = number(&amount.buy_amount, "buyAmount")?;
    let sell = number(&amount.sell_amount, "sellAmount")?;
    let net = buy - sell;
    Ok(NewObservation {
        source: "tossinvest".into(),
        series: format!("{market}_{category}_NET_BUY_KRW"),
        entity: market.into(),
        observed_at: date.into(),
        value: net,
        released_at: Some(updated_at.into()),
        source_asof: Some(updated_at.into()),
        revision_id: Some(format!("{updated_at}:{net:.0}")),
        metadata: json!({
            "provider":"Toss Securities Open API",
            "endpoint":format!("/api/v1/market-indicators/{market}/investor-trading"),
            "official":true,
            "category":category,
            "buyAmount":buy,
            "sellAmount":sell,
            "netBuyAmount":net,
            "currency":"KRW"
        }),
    })
}

fn store_investor_record(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    record: &InvestorTradingRecord,
) {
    let institution = TradingAmount {
        buy_amount: record.institution.buy_amount.clone(),
        sell_amount: record.institution.sell_amount.clone(),
    };
    let entries = [
        ("INDIVIDUAL", &record.individual),
        ("FOREIGNER", &record.foreigner),
        ("INSTITUTION", &institution),
        ("OTHER_CORPORATION", &record.other_corporation),
        (
            "FINANCIAL_INVESTMENT",
            &record.institution.breakdown.financial_investment,
        ),
        ("INSURANCE", &record.institution.breakdown.insurance),
        ("TRUST", &record.institution.breakdown.trust),
        (
            "PRIVATE_EQUITY_FUND",
            &record.institution.breakdown.private_equity_fund,
        ),
        ("BANK", &record.institution.breakdown.bank),
        (
            "OTHER_FINANCIAL_INSTITUTION",
            &record.institution.breakdown.other_financial_institution,
        ),
        ("PENSION_FUND", &record.institution.breakdown.pension_fund),
    ];
    for (category, amount) in entries {
        match amount_observation(market, category, &record.date, &record.updated_at, amount) {
            Ok(observation) => store_observation(db, report, observation),
            Err(error) => {
                report.attempted += 1;
                report
                    .errors
                    .push(format!("toss {market} {category}: {error}"));
            }
        }
    }
}

fn store_payload(
    db: &Db,
    report: &mut CollectionReport,
    key: &str,
    observed_at: &str,
    payload: &Value,
) {
    let result = db
        .put_market_payload(key, observed_at, payload)
        .map_err(|error| format!("database: {error}"));
    record_result(report, result, key);
}

fn store_daily_candles(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    page: &MarketCandlePage,
) {
    for candle in &page.candles {
        for (suffix, raw, field) in [
            ("OPEN", candle.open_price.as_str(), "openPrice"),
            ("HIGH", candle.high_price.as_str(), "highPrice"),
            ("LOW", candle.low_price.as_str(), "lowPrice"),
            ("CLOSE", candle.close_price.as_str(), "closePrice"),
            ("VOLUME", candle.volume.as_str(), "volume"),
        ] {
            let value = match number(raw, field) {
                Ok(value) => value,
                Err(error) => {
                    report.attempted += 1;
                    report
                        .errors
                        .push(format!("toss {market} candle {field}: {error}"));
                    continue;
                }
            };
            store_observation(
                db,
                report,
                NewObservation {
                    source: "tossinvest".into(),
                    series: format!("{market}_DAY_{suffix}"),
                    entity: market.into(),
                    observed_at: candle.timestamp.clone(),
                    value,
                    released_at: None,
                    source_asof: None,
                    revision_id: Some(format!("{}:{field}:{raw}", candle.timestamp)),
                    metadata: json!({
                        "provider":"Toss Securities Open API",
                        "endpoint":format!("/api/v1/market-indicators/{market}/candles"),
                        "official":true,
                        "interval":"1d",
                        "field":field
                    }),
                },
            );
        }
    }
}

fn collect_calendar(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    db: &Db,
    report: &mut CollectionReport,
    country: &str,
) {
    let path = format!("/api/v1/market-calendar/{country}");
    match authenticated_json(http, client_id, client_secret, &path, &[])
        .and_then(|value| result_value(value, &path))
    {
        Ok(result) => {
            let observed_at = Utc::now().to_rfc3339();
            store_payload(
                db,
                report,
                &format!("toss.calendar.{country}"),
                &observed_at,
                &result,
            );
        }
        Err(error) => report
            .errors
            .push(format!("toss calendar {country}: {error}")),
    }
}

fn collect_investors(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
) {
    let path = format!("/api/v1/market-indicators/{market}/investor-trading");
    let query = [("interval", "1d".to_string()), ("count", "30".to_string())];
    match authenticated_json(http, client_id, client_secret, &path, &query)
        .and_then(|value| result_value(value, &path))
    {
        Ok(result) => match serde_json::from_value::<InvestorTradingResponse>(result.clone()) {
            Ok(parsed) => {
                for record in &parsed.records {
                    store_investor_record(db, report, market, record);
                }
                let observed_at = parsed
                    .records
                    .first()
                    .map(|record| record.updated_at.clone())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                store_payload(
                    db,
                    report,
                    &format!("toss.investor.{market}"),
                    &observed_at,
                    &result,
                );
            }
            Err(error) => report
                .errors
                .push(format!("toss investor {market}: invalid response: {error}")),
        },
        Err(error) => report
            .errors
            .push(format!("toss investor {market}: {error}")),
    }
}

fn collect_candles(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
) {
    let path = format!("/api/v1/market-indicators/{market}/candles");
    let query = [("interval", "1d".to_string()), ("count", "30".to_string())];
    match authenticated_json(http, client_id, client_secret, &path, &query)
        .and_then(|value| result_value(value, &path))
    {
        Ok(result) => match serde_json::from_value::<MarketCandlePage>(result) {
            Ok(page) => store_daily_candles(db, report, market, &page),
            Err(error) => report
                .errors
                .push(format!("toss candle {market}: invalid response: {error}")),
        },
        Err(error) => report.errors.push(format!("toss candle {market}: {error}")),
    }
}

fn fetch_stock_names(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    symbols: &BTreeSet<String>,
) -> Result<BTreeMap<String, StockInfo>, String> {
    if symbols.is_empty() {
        return Ok(BTreeMap::new());
    }
    let joined = symbols.iter().cloned().collect::<Vec<_>>().join(",");
    let value = authenticated_json(
        http,
        client_id,
        client_secret,
        "/api/v1/stocks",
        &[("symbols", joined)],
    )?;
    let result = result_value(value, "/api/v1/stocks")?;
    let stocks = serde_json::from_value::<Vec<StockInfo>>(result)
        .map_err(|error| format!("/api/v1/stocks: invalid response: {error}"))?;
    Ok(stocks
        .into_iter()
        .map(|stock| (stock.symbol.clone(), stock))
        .collect())
}

fn collect_rankings(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    db: &Db,
    report: &mut CollectionReport,
) {
    let specifications = [
        ("KR", "turnover", "MARKET_TRADING_AMOUNT", "realtime"),
        ("KR", "gainers", "TOP_GAINERS", "1d"),
        ("KR", "losers", "TOP_LOSERS", "1d"),
        ("US", "turnover", "MARKET_TRADING_AMOUNT", "realtime"),
        ("US", "gainers", "TOP_GAINERS", "1d"),
        ("US", "losers", "TOP_LOSERS", "1d"),
    ];
    let mut pages = Vec::new();
    let mut symbols = BTreeSet::new();
    for (index, (country, key, ranking_type, duration)) in specifications.iter().enumerate() {
        if index > 0 {
            thread::sleep(Duration::from_millis(225));
        }
        let query = [
            ("type", (*ranking_type).to_string()),
            ("marketCountry", (*country).to_string()),
            ("duration", (*duration).to_string()),
            ("excludeInvestmentCaution", "true".to_string()),
            ("count", "10".to_string()),
        ];
        match authenticated_json(http, client_id, client_secret, "/api/v1/rankings", &query)
            .and_then(|value| result_value(value, "/api/v1/rankings"))
        {
            Ok(result) => match serde_json::from_value::<RankingResponse>(result) {
                Ok(page) => {
                    symbols.extend(page.rankings.iter().map(|item| item.symbol.clone()));
                    pages.push((country.to_string(), key.to_string(), page));
                }
                Err(error) => report.errors.push(format!(
                    "toss ranking {country} {key}: invalid response: {error}"
                )),
            },
            Err(error) => report
                .errors
                .push(format!("toss ranking {country} {key}: {error}")),
        }
    }
    let names = match fetch_stock_names(http, client_id, client_secret, &symbols) {
        Ok(names) => names,
        Err(error) => {
            report
                .errors
                .push(format!("toss ranking stock names: {error}"));
            BTreeMap::new()
        }
    };
    for (country, key, mut page) in pages {
        for item in &mut page.rankings {
            if let Some(stock) = names.get(&item.symbol) {
                item.name = Some(stock.name.clone());
                item.market = Some(stock.market.clone());
                item.security_type = Some(stock.security_type.clone());
            }
        }
        let observed_at = page
            .ranked_at
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let payload = serde_json::to_value(&page).unwrap_or_else(|_| json!({"rankings":[]}));
        store_payload(
            db,
            report,
            &format!("toss.ranking.{country}.{key}"),
            &observed_at,
            &payload,
        );
    }
}

pub fn collect_realtime(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let mut report = CollectionReport::default();
    let (client_id, client_secret) = match credentials(config) {
        Ok(credentials) => credentials,
        Err(error) => {
            report.errors.push(format!("toss credentials: {error}"));
            return Ok(report);
        }
    };
    let http = http_client(config)?;
    let price_query = [("symbols", MARKET_SYMBOLS.to_string())];
    match authenticated_json(
        &http,
        client_id,
        client_secret,
        "/api/v1/market-indicators/prices",
        &price_query,
    ) {
        Ok(value) => match serde_json::from_value::<MarketIndicatorResponse>(value) {
            Ok(response) => store_prices(db, &mut report, &response.result),
            Err(error) => report
                .errors
                .push(format!("toss market prices: invalid response: {error}")),
        },
        Err(error) => report.errors.push(format!("toss market prices: {error}")),
    }
    let fx_query = [
        ("baseCurrency", "USD".to_string()),
        ("quoteCurrency", "KRW".to_string()),
    ];
    match authenticated_json(
        &http,
        client_id,
        client_secret,
        "/api/v1/exchange-rate",
        &fx_query,
    )
    .and_then(|value| result_value(value, "/api/v1/exchange-rate"))
    {
        Ok(result) => match serde_json::from_value::<ExchangeRate>(result) {
            Ok(rate) => store_exchange_rate(db, &mut report, &rate),
            Err(error) => report
                .errors
                .push(format!("toss exchange rate: invalid response: {error}")),
        },
        Err(error) => report.errors.push(format!("toss exchange rate: {error}")),
    }
    Ok(report)
}

pub fn collect_market_details(
    config: &Config,
    db: &Db,
) -> Result<CollectionReport, Box<dyn Error>> {
    let mut report = CollectionReport::default();
    let (client_id, client_secret) = match credentials(config) {
        Ok(credentials) => credentials,
        Err(error) => {
            report.errors.push(format!("toss credentials: {error}"));
            return Ok(report);
        }
    };
    let http = http_client(config)?;
    for country in ["KR", "US"] {
        collect_calendar(&http, client_id, client_secret, db, &mut report, country);
    }
    for market in ["KOSPI", "KOSDAQ"] {
        collect_investors(&http, client_id, client_secret, db, &mut report, market);
        collect_candles(&http, client_id, client_secret, db, &mut report, market);
    }
    collect_rankings(&http, client_id, client_secret, db, &mut report);
    Ok(report)
}

pub fn collect_all(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let mut report = collect_realtime(config, db)?;
    if config.has_tossinvest_credentials() {
        report.merge(collect_market_details(config, db)?);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_market_indicator_example_parses_all_catalog_types() {
        let response: MarketIndicatorResponse = serde_json::from_value(json!({
            "result":[
                {"symbol":"KOSPI","timestamp":"2026-06-11T15:30:00+09:00","lastPrice":"2812.45"},
                {"symbol":"KR_BOND_10Y","timestamp":"2026-06-11T15:29:58+09:00","lastPrice":"3.25"}
            ]
        }))
        .unwrap();
        assert_eq!(response.result.len(), 2);
        assert_eq!(response.result[1].last_price, "3.25");
    }

    #[test]
    fn official_exchange_rate_example_parses() {
        let value: ExchangeRate = serde_json::from_value(json!({
            "baseCurrency":"USD","quoteCurrency":"KRW","rate":"1380.5",
            "midRate":"1375","basisPoint":"40","rateChangeType":"UP",
            "validFrom":"2026-03-25T09:30:00+09:00",
            "validUntil":"2026-03-25T09:31:00+09:00"
        }))
        .unwrap();
        assert_eq!(value.mid_rate, "1375");
    }

    #[test]
    fn official_investor_example_keeps_buy_and_sell_originals() {
        let value: InvestorTradingResponse = serde_json::from_value(json!({
            "records":[{
                "date":"2026-06-11","updatedAt":"2026-06-11T18:10:00+09:00",
                "individual":{"buyAmount":"520","sellAmount":"535"},
                "foreigner":{"buyAmount":"380","sellAmount":"360"},
                "institution":{"buyAmount":"210","sellAmount":"218","breakdown":{
                    "financialInvestment":{"buyAmount":"90","sellAmount":"95"},
                    "insurance":{"buyAmount":"15","sellAmount":"14"},
                    "trust":{"buyAmount":"28","sellAmount":"30"},
                    "privateEquityFund":{"buyAmount":"12","sellAmount":"13"},
                    "bank":{"buyAmount":"5","sellAmount":"6"},
                    "otherFinancialInstitution":{"buyAmount":"10","sellAmount":"11"},
                    "pensionFund":{"buyAmount":"50","sellAmount":"49"}
                }},
                "otherCorporation":{"buyAmount":"45","sellAmount":"42"}
            }]
        }))
        .unwrap();
        assert_eq!(value.records[0].foreigner.buy_amount, "380");
        assert_eq!(
            value.records[0]
                .institution
                .breakdown
                .pension_fund
                .sell_amount,
            "49"
        );
    }

    #[test]
    fn source_timestamp_preserves_korea_offset() {
        assert_eq!(
            normalized_timestamp("2026-06-11T15:30:00+09:00").unwrap(),
            "2026-06-11T15:30:00+09:00"
        );
    }

    #[test]
    fn ranking_round_trip_preserves_enrichment() {
        let mut page: RankingResponse = serde_json::from_value(json!({
            "rankedAt":"2026-06-10T14:30:00+09:00",
            "rankings":[{"rank":1,"symbol":"005930","currency":"KRW",
                "price":{"lastPrice":"56500","basePrice":"55800","changeRate":"0.0125"},
                "tradingVolume":"18432100","tradingAmount":"1041436650000"}]
        }))
        .unwrap();
        page.rankings[0].name = Some("삼성전자".into());
        let serialized = serde_json::to_value(page).unwrap();
        assert_eq!(serialized["rankings"][0]["name"], "삼성전자");
    }

    #[test]
    fn concise_errors_never_include_credentials() {
        let value = json!({"error":{"code":"invalid-request","message":"bad request"}});
        assert_eq!(
            concise_error(StatusCode::BAD_REQUEST, Some(value)),
            "HTTP 400 Bad Request invalid-request: bad request"
        );
    }
}
