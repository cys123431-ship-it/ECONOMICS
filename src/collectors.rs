use crate::{
    config::Config,
    db::{Db, NewObservation},
};
use chrono::{SecondsFormat, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error, fs, time::Duration};

type AuctionValues = (f64, Option<f64>, Option<f64>, Option<f64>);

#[derive(Clone, Debug, Default, Serialize)]
pub struct CollectionReport {
    pub attempted: usize,
    pub stored: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

impl CollectionReport {
    pub fn merge(&mut self, other: Self) {
        self.attempted += other.attempted;
        self.stored += other.stored;
        self.unchanged += other.unchanged;
        self.errors.extend(other.errors);
    }

    fn record(&mut self, result: rusqlite::Result<bool>) {
        self.attempted += 1;
        match result {
            Ok(true) => self.stored += 1,
            Ok(false) => self.unchanged += 1,
            Err(error) => self.errors.push(format!("database: {error}")),
        }
    }

    fn error(&mut self, source: &str, error: impl std::fmt::Display) {
        self.errors.push(format!("{source}: {error}"));
    }

    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

fn client(config: &Config) -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(config.http_timeout_secs))
        .user_agent("ECONOMICS-Radar/0.3")
        .build()?)
}

const FRED_SERIES: &[&str] = &[
    "WEI",
    "CFNAI",
    "SAHMREALTIME",
    "ICSA",
    "CCSA",
    "STLFSI4",
    "NFCI",
    "ANFCI",
    "NFCILEVERAGE",
    "BAMLH0A0HYM2",
    "BAMLC0A0CM",
    "T10Y2Y",
    "T10Y3M",
    "VIXCLS",
    "WALCL",
    "RRPONTSYD",
    "TOTRESNS",
    "WRESBAL",
    "DEXKOUS",
    "DTWEXBGS",
    "MORTGAGE30US",
    "DRCCLACBS",
    "DRCLACBS",
    "BUSLOANS",
    "TOTLL",
    "EQTA",
    "KORLOLITOAASTSAM",
    "CHNLOLITOAASTSAM",
];

pub fn collect_fred(
    config: &Config,
    db: &Db,
    start: &str,
    initial_release: bool,
) -> Result<CollectionReport, Box<dyn Error>> {
    let key = config.fred_api_key.as_ref().ok_or("FRED_API_KEY missing")?;
    let http = client(config)?;
    let mut report = CollectionReport::default();

    for series in FRED_SERIES {
        let mut url = format!(
            "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json&observation_start={}&limit=100000",
            urlencoding::encode(series),
            urlencoding::encode(key),
            urlencoding::encode(start)
        );
        if initial_release {
            url.push_str("&output_type=4");
        }
        let response = http
            .get(&url)
            .send()
            .and_then(|response| response.error_for_status());
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                report.error(series, redact_url_error(error.to_string(), key));
                continue;
            }
        };
        let value: Value = match response.json() {
            Ok(value) => value,
            Err(error) => {
                report.error(series, error);
                continue;
            }
        };
        let Some(observations) = value.get("observations").and_then(Value::as_array) else {
            report.error(series, "observations missing in response");
            continue;
        };
        for observation in observations {
            let Some(date) = observation.get("date").and_then(Value::as_str) else {
                continue;
            };
            let Some(number) = parse_number(observation.get("value")) else {
                continue;
            };
            let released_at = observation
                .get("realtime_start")
                .and_then(Value::as_str)
                .map(date_to_end_of_day);
            let source_asof = observation
                .get("realtime_end")
                .and_then(Value::as_str)
                .map(date_to_end_of_day);
            let revision_id = released_at
                .clone()
                .unwrap_or_else(|| format!("value:{number:.17}"));
            report.record(db.put(&NewObservation {
                source: if initial_release { "alfred" } else { "fred" }.into(),
                series: (*series).into(),
                entity: String::new(),
                observed_at: date.into(),
                value: number,
                released_at,
                source_asof,
                revision_id: Some(revision_id),
                metadata: json!({"initial_release": initial_release}),
            }));
        }
    }
    Ok(report)
}

pub fn collect_treasury(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let http = client(config)?;
    let url = "https://api.fiscaldata.treasury.gov/services/api/fiscal_service/v1/accounting/od/auctions_query?sort=-auction_date&page%5Bsize%5D=500";
    let value: Value = http.get(url).send()?.error_for_status()?.json()?;
    let rows = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or("Treasury response missing data")?;
    let mut report = CollectionReport::default();
    let mut daily: BTreeMap<String, Vec<AuctionValues>> = BTreeMap::new();

    for row in rows {
        let Some(date) = row.get("auction_date").and_then(Value::as_str) else {
            continue;
        };
        let entity = row
            .get("cusip")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let bid_to_cover = parse_number(row.get("bid_to_cover_ratio"));
        let offering = parse_number(row.get("offering_amt"));
        let dealer = ratio(parse_number(row.get("primary_dealer_accepted")), offering);
        let direct = ratio(parse_number(row.get("direct_bidder_accepted")), offering);
        let indirect = ratio(parse_number(row.get("indirect_bidder_accepted")), offering);
        let source_asof = row
            .get("record_date")
            .and_then(Value::as_str)
            .map(date_to_end_of_day);
        let metadata = json!({
            "cusip": entity,
            "security_type": row.get("security_type"),
            "security_term": row.get("security_term"),
            "offering_amount": offering
        });
        for (series, value) in [
            ("AUCTION_BTC_RAW", bid_to_cover),
            ("AUCTION_DEALER_SHARE_RAW", dealer),
            ("AUCTION_DIRECT_SHARE_RAW", direct),
            ("AUCTION_INDIRECT_SHARE_RAW", indirect),
        ] {
            if let Some(value) = value {
                report.record(db.put(&NewObservation {
                    source: "treasury".into(),
                    series: series.into(),
                    entity: entity.into(),
                    observed_at: date.into(),
                    value,
                    released_at: Some(date_to_end_of_day(date)),
                    source_asof: source_asof.clone(),
                    revision_id: Some(format!(
                        "{}:{value:.17}",
                        source_asof.as_deref().unwrap_or(date)
                    )),
                    metadata: metadata.clone(),
                }));
            }
        }
        if let Some(bid_to_cover) = bid_to_cover {
            daily
                .entry(date.into())
                .or_default()
                .push((bid_to_cover, dealer, direct, indirect));
        }
    }

    for (date, auctions) in daily {
        let aggregate = |index: usize| {
            let values = auctions
                .iter()
                .filter_map(|values| match index {
                    0 => Some(values.0),
                    1 => values.1,
                    2 => values.2,
                    _ => values.3,
                })
                .collect::<Vec<_>>();
            (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
        };
        for (series, value) in [
            ("AUCTION_BTC", aggregate(0)),
            ("AUCTION_DEALER_SHARE", aggregate(1)),
            ("AUCTION_DIRECT_SHARE", aggregate(2)),
            ("AUCTION_INDIRECT_SHARE", aggregate(3)),
        ] {
            if let Some(value) = value {
                report.record(db.put(&NewObservation {
                    source: "treasury".into(),
                    series: series.into(),
                    entity: String::new(),
                    observed_at: date.clone(),
                    value,
                    released_at: Some(date_to_end_of_day(&date)),
                    source_asof: None,
                    revision_id: Some(format!("aggregate:{}:{value:.17}", auctions.len())),
                    metadata: json!({"auction_count": auctions.len()}),
                }));
            }
        }
    }
    Ok(report)
}

pub fn collect_binance(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let http = client(config)?;
    let mut report = CollectionReport::default();
    for request in [
        BinanceRequest::array(
            "funding",
            "https://fapi.binance.com/fapi/v1/fundingRate?symbol=BTCUSDT&limit=1000",
            "BTC_FUNDING_ABS",
            "fundingRate",
            "fundingTime",
            true,
        ),
        BinanceRequest::array(
            "open-interest-history",
            "https://fapi.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=1h&limit=500",
            "BTC_OI",
            "sumOpenInterestValue",
            "timestamp",
            false,
        ),
        BinanceRequest::array(
            "global-long-short",
            "https://fapi.binance.com/futures/data/globalLongShortAccountRatio?symbol=BTCUSDT&period=1h&limit=500",
            "BTC_GLOBAL_LONG_SHORT",
            "longShortRatio",
            "timestamp",
            false,
        ),
        BinanceRequest::array(
            "top-position",
            "https://fapi.binance.com/futures/data/topLongShortPositionRatio?symbol=BTCUSDT&period=1h&limit=500",
            "BTC_TOP_POSITION_RATIO",
            "longShortRatio",
            "timestamp",
            false,
        ),
        BinanceRequest::array(
            "top-account",
            "https://fapi.binance.com/futures/data/topLongShortAccountRatio?symbol=BTCUSDT&period=1h&limit=500",
            "BTC_TOP_ACCOUNT_RATIO",
            "longShortRatio",
            "timestamp",
            false,
        ),
        BinanceRequest::array(
            "taker-ratio",
            "https://fapi.binance.com/futures/data/takerlongshortRatio?symbol=BTCUSDT&period=1h&limit=500",
            "BTC_TAKER_RATIO",
            "buySellRatio",
            "timestamp",
            false,
        ),
        BinanceRequest::array(
            "basis",
            "https://fapi.binance.com/futures/data/basis?pair=BTCUSDT&contractType=PERPETUAL&period=1h&limit=500",
            "BTC_BASIS_ABS",
            "basisRate",
            "timestamp",
            true,
        ),
    ] {
        match http.get(request.url).send().and_then(|response| response.error_for_status()) {
            Ok(response) => match response.json::<Value>() {
                Ok(value) => store_binance_array(db, &mut report, &request, &value),
                Err(error) => report.error(request.name, error),
            },
            Err(error) => report.error(request.name, error),
        }
    }
    Ok(report)
}

struct BinanceRequest {
    name: &'static str,
    url: &'static str,
    series: &'static str,
    value_field: &'static str,
    time_field: &'static str,
    absolute: bool,
}

impl BinanceRequest {
    const fn array(
        name: &'static str,
        url: &'static str,
        series: &'static str,
        value_field: &'static str,
        time_field: &'static str,
        absolute: bool,
    ) -> Self {
        Self {
            name,
            url,
            series,
            value_field,
            time_field,
            absolute,
        }
    }
}

fn store_binance_array(
    db: &Db,
    report: &mut CollectionReport,
    request: &BinanceRequest,
    value: &Value,
) {
    let Some(rows) = value.as_array() else {
        report.error(request.name, "expected JSON array");
        return;
    };
    for row in rows {
        let Some(mut number) = parse_number(row.get(request.value_field)) else {
            continue;
        };
        if request.absolute {
            number = number.abs();
        }
        let Some(timestamp) = parse_millis(row.get(request.time_field)) else {
            continue;
        };
        report.record(db.put(&NewObservation {
            source: "binance".into(),
            series: request.series.into(),
            entity: "BTCUSDT".into(),
            observed_at: timestamp.clone(),
            value: number,
            released_at: Some(timestamp.clone()),
            source_asof: Some(timestamp.clone()),
            revision_id: Some(format!("{timestamp}:{number:.17}")),
            metadata: json!({"symbol":"BTCUSDT","endpoint":request.name}),
        }));
    }
}

pub fn collect_ecos(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let key = config.ecos_api_key.as_ref().ok_or("ECOS_API_KEY missing")?;
    let http = client(config)?;
    let mut report = CollectionReport::default();
    for (series, stat, item) in [
        ("KR_BASE_RATE", "722Y001", "0101000"),
        ("KR_USD_KRW", "731Y001", "0000001"),
    ] {
        let end = Utc::now().format("%Y%m%d").to_string();
        let page_size = 1_000usize;
        let mut start_row = 1usize;
        let mut total = usize::MAX;
        while start_row <= total {
            let end_row = start_row + page_size - 1;
            let url =
                format!(
                "https://ecos.bok.or.kr/api/StatisticSearch/{}/json/kr/{}/{}/{}/D/20000101/{}/{}",
                urlencoding::encode(key), start_row, end_row, stat, end, item
            );
            let value: Value = match http.get(&url).send().and_then(|r| r.error_for_status()) {
                Ok(response) => match response.json() {
                    Ok(value) => value,
                    Err(error) => {
                        report.error(series, error);
                        break;
                    }
                },
                Err(error) => {
                    report.error(series, redact_url_error(error.to_string(), key));
                    break;
                }
            };
            total = value
                .pointer("/StatisticSearch/list_total_count")
                .and_then(|value| {
                    value
                        .as_u64()
                        .or_else(|| value.as_str()?.parse::<u64>().ok())
                })
                .unwrap_or(0) as usize;
            let Some(rows) = value
                .pointer("/StatisticSearch/row")
                .and_then(Value::as_array)
            else {
                if total != 0 {
                    report.error(series, "ECOS row missing");
                }
                break;
            };
            for row in rows {
                let Some(date) = row.get("TIME").and_then(Value::as_str) else {
                    continue;
                };
                let Some(number) = parse_number(row.get("DATA_VALUE")) else {
                    continue;
                };
                let observed_at = normalize_compact_date(date);
                report.record(db.put(&NewObservation {
                    source: "ecos".into(),
                    series: series.into(),
                    entity: String::new(),
                    observed_at,
                    value: number,
                    released_at: None,
                    source_asof: Some(Utc::now().to_rfc3339()),
                    revision_id: Some(format!("value:{number:.17}")),
                    metadata: json!({"stat_code":stat,"item_code":item}),
                }));
            }
            if rows.len() < page_size {
                break;
            }
            start_row += page_size;
        }
    }
    Ok(report)
}

pub fn collect_krx(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let Some(url) = config.krx_api_url.as_ref() else {
        return Ok(CollectionReport::default());
    };
    let key = config.krx_api_key.as_ref().ok_or("KRX_API_KEY missing")?;
    let value: Value = client(config)?
        .get(url)
        .header("AUTH_KEY", key)
        .send()?
        .error_for_status()?
        .json()?;
    let rows = find_object_rows(&value).ok_or("KRX JSON contains no row array")?;
    let aliases: &[(&str, &[&str])] = &[
        (
            "KRX_FOREIGN_NET_BUY",
            &["FORN_NET_BUY", "FOREIGN_NET_BUY", "FRGN_NET_BUY"],
        ),
        (
            "KRX_SHORT_BALANCE",
            &["SHORT_BALANCE", "SHORT_BAL", "SRTSAL_BAL"],
        ),
        ("KRX_FUTURES_BASIS", &["FUTURES_BASIS", "BASIS"]),
        ("KRX_PUT_CALL_RATIO", &["PUT_CALL_RATIO", "P_C_RATIO"]),
        ("KRX_ADVANCES", &["ADVANCES", "UP_ISSUES"]),
        ("KRX_DECLINES", &["DECLINES", "DOWN_ISSUES"]),
    ];
    let mut report = CollectionReport::default();
    for row in rows {
        let date = ["TRD_DD", "BAS_DD", "DATE", "date"]
            .iter()
            .find_map(|field| row.get(*field).and_then(Value::as_str))
            .map(normalize_compact_date)
            .unwrap_or_else(|| Utc::now().date_naive().to_string());
        for (series, fields) in aliases {
            if let Some(number) = fields
                .iter()
                .find_map(|field| parse_number(row.get(*field)))
            {
                report.record(db.put(&NewObservation {
                    source: "krx".into(),
                    series: (*series).into(),
                    entity: String::new(),
                    observed_at: date.clone(),
                    value: number,
                    released_at: Some(date_to_end_of_day(&date)),
                    source_asof: Some(Utc::now().to_rfc3339()),
                    revision_id: Some(format!("value:{number:.17}")),
                    metadata: Value::Null,
                }));
            }
        }
        let advances = ["ADVANCES", "UP_ISSUES"]
            .iter()
            .find_map(|field| parse_number(row.get(*field)));
        let declines = ["DECLINES", "DOWN_ISSUES"]
            .iter()
            .find_map(|field| parse_number(row.get(*field)));
        if let (Some(advances), Some(declines)) = (advances, declines) {
            if advances + declines > f64::EPSILON {
                let breadth = 100.0 * advances / (advances + declines);
                report.record(db.put(&NewObservation {
                    source: "krx".into(),
                    series: "KRX_BREADTH".into(),
                    entity: String::new(),
                    observed_at: date.clone(),
                    value: breadth,
                    released_at: Some(date_to_end_of_day(&date)),
                    source_asof: Some(Utc::now().to_rfc3339()),
                    revision_id: Some(format!("value:{breadth:.17}")),
                    metadata: json!({"advances":advances,"declines":declines}),
                }));
            }
        }
    }
    if report.attempted == 0 {
        report.error(
            "krx",
            "no recognized market-internals fields in configured response",
        );
    }
    Ok(report)
}

#[derive(Debug, Deserialize)]
struct AdapterFile {
    adapters: Vec<OfficialAdapter>,
}

#[derive(Debug, Deserialize)]
struct OfficialAdapter {
    source: String,
    url: String,
    records_pointer: String,
    date_field: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    entity_field: Option<String>,
    #[serde(default)]
    released_field: Option<String>,
    #[serde(default)]
    source_asof_field: Option<String>,
    series: Vec<AdapterSeries>,
}

#[derive(Debug, Deserialize)]
struct AdapterSeries {
    name: String,
    value_field: String,
    #[serde(default)]
    absolute: bool,
}

pub fn collect_configured_adapters(
    config: &Config,
    db: &Db,
) -> Result<CollectionReport, Box<dyn Error>> {
    let Some(path) = config.official_adapters_file.as_ref() else {
        return Ok(CollectionReport::default());
    };
    let adapters: AdapterFile = serde_json::from_str(&fs::read_to_string(path)?)?;
    let http = client(config)?;
    let mut report = CollectionReport::default();
    for adapter in adapters.adapters {
        if !adapter.enabled {
            continue;
        }
        let value: Value = match http
            .get(&adapter.url)
            .send()
            .and_then(|response| response.error_for_status())
        {
            Ok(response) => match response.json() {
                Ok(value) => value,
                Err(error) => {
                    report.error(&adapter.source, error);
                    continue;
                }
            },
            Err(error) => {
                report.error(&adapter.source, error);
                continue;
            }
        };
        let Some(rows) = value
            .pointer(&adapter.records_pointer)
            .and_then(Value::as_array)
        else {
            report.error(
                &adapter.source,
                "records_pointer did not resolve to an array",
            );
            continue;
        };
        for row in rows {
            let Some(date) = row.get(&adapter.date_field).and_then(Value::as_str) else {
                continue;
            };
            let entity = adapter
                .entity_field
                .as_ref()
                .and_then(|field| row.get(field))
                .and_then(Value::as_str)
                .unwrap_or("");
            let released_at = adapter
                .released_field
                .as_ref()
                .and_then(|field| row.get(field))
                .and_then(Value::as_str)
                .map(date_to_end_of_day);
            let source_asof = adapter
                .source_asof_field
                .as_ref()
                .and_then(|field| row.get(field))
                .and_then(Value::as_str)
                .map(date_to_end_of_day)
                .or_else(|| Some(Utc::now().to_rfc3339()));
            for series in &adapter.series {
                let Some(mut number) = parse_number(row.get(&series.value_field)) else {
                    continue;
                };
                if series.absolute {
                    number = number.abs();
                }
                report.record(db.put(&NewObservation {
                    source: adapter.source.clone(),
                    series: series.name.clone(),
                    entity: entity.into(),
                    observed_at: normalize_compact_date(date),
                    value: number,
                    released_at: released_at.clone(),
                    source_asof: source_asof.clone(),
                    revision_id: Some(format!("value:{number:.17}")),
                    metadata: Value::Null,
                }));
            }
        }
    }
    Ok(report)
}

fn default_true() -> bool {
    true
}

fn ratio(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator.abs() > f64::EPSILON => {
            Some(numerator / denominator)
        }
        _ => None,
    }
}

fn parse_number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(number) => number.replace(',', "").parse().ok(),
        _ => None,
    }
}

fn parse_millis(value: Option<&Value>) -> Option<String> {
    let millis = match value? {
        Value::Number(value) => value.as_i64()?,
        Value::String(value) => value.parse().ok()?,
        _ => return None,
    };
    chrono::DateTime::from_timestamp_millis(millis)
        .map(|date| date.to_rfc3339_opts(SecondsFormat::Millis, true))
}

fn date_to_end_of_day(date: &str) -> String {
    if date.contains('T') {
        date.into()
    } else {
        format!("{}T23:59:59Z", normalize_compact_date(date))
    }
}

fn normalize_compact_date(date: &str) -> String {
    let digits = date
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    if digits.len() == 8 {
        format!("{}-{}-{}", &digits[..4], &digits[4..6], &digits[6..8])
    } else {
        date.into()
    }
}

fn redact_url_error(mut message: String, secret: &str) -> String {
    if !secret.is_empty() {
        message = message.replace(secret, "[REDACTED]");
    }
    message
}

fn find_object_rows(value: &Value) -> Option<&Vec<Value>> {
    match value {
        Value::Array(rows) if rows.iter().any(Value::is_object) => Some(rows),
        Value::Array(rows) => rows.iter().find_map(find_object_rows),
        Value::Object(map) => map.values().find_map(find_object_rows),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_dates_are_normalized() {
        assert_eq!(normalize_compact_date("20260820"), "2026-08-20");
        assert_eq!(normalize_compact_date("2026-08-20"), "2026-08-20");
    }

    #[test]
    fn ratios_reject_zero_denominators() {
        assert_eq!(ratio(Some(10.0), Some(20.0)), Some(0.5));
        assert_eq!(ratio(Some(10.0), Some(0.0)), None);
    }

    #[test]
    fn nested_row_arrays_are_discovered() {
        let value = json!({"response":{"data":[{"DATE":"20260820","BASIS":"1.2"}]}});
        assert_eq!(find_object_rows(&value).unwrap().len(), 1);
    }
}
