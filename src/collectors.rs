use crate::{
    config::Config,
    db::{Db, NewObservation},
};
use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, SecondsFormat, Utc, Weekday};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error, fs, thread, time::Duration};

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

const FRED_VINTAGE_PAGE_SIZE: usize = 10_000;
const FRED_VINTAGE_WINDOW_SIZE: usize = 100;
const FRED_REQUEST_ATTEMPTS: usize = 3;

fn fred_vintage_windows(dates: &[String]) -> Vec<(String, String)> {
    dates
        .chunks(FRED_VINTAGE_WINDOW_SIZE)
        .filter_map(|chunk| Some((chunk.first()?.clone(), chunk.last()?.clone())))
        .collect()
}

fn fred_json(
    http: &Client,
    endpoint: &str,
    query: &[(&str, String)],
    key: &str,
) -> Result<Value, String> {
    for attempt in 0..FRED_REQUEST_ATTEMPTS {
        match http.get(endpoint).query(query).send() {
            Ok(response) if response.status().is_success() => {
                return response.json().map_err(|error| error.to_string());
            }
            Ok(response) => {
                let status = response.status();
                let retryable = status.is_server_error() || status.as_u16() == 429;
                let error = response
                    .error_for_status()
                    .expect_err("non-success FRED response must have an HTTP error");
                if !retryable || attempt + 1 == FRED_REQUEST_ATTEMPTS {
                    return Err(redact_url_error(error.to_string(), key));
                }
            }
            Err(error) => {
                if attempt + 1 == FRED_REQUEST_ATTEMPTS {
                    return Err(redact_url_error(error.to_string(), key));
                }
            }
        }
        thread::sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
    }
    Err("FRED request exhausted retries".into())
}

fn fred_revision_id(initial_release: bool, released_at: Option<&str>, value: f64) -> String {
    if initial_release {
        released_at
            .map(str::to_string)
            .unwrap_or_else(|| format!("initial:{value:.17}"))
    } else {
        format!("current:{value:.17}")
    }
}

fn fred_vintage_dates(
    http: &Client,
    key: &str,
    series: &str,
    start: &str,
) -> Result<Vec<String>, String> {
    let mut dates = Vec::new();
    let mut offset = 0usize;
    loop {
        let query = [
            ("series_id", series.to_string()),
            ("api_key", key.to_string()),
            ("file_type", "json".to_string()),
            ("realtime_start", start.to_string()),
            ("realtime_end", Utc::now().date_naive().to_string()),
            ("limit", FRED_VINTAGE_PAGE_SIZE.to_string()),
            ("offset", offset.to_string()),
        ];
        let value = fred_json(
            http,
            "https://api.stlouisfed.org/fred/series/vintagedates",
            &query,
            key,
        )?;
        let page = value
            .get("vintage_dates")
            .and_then(Value::as_array)
            .ok_or_else(|| "vintage_dates missing in response".to_string())?;
        let page_dates = page
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let page_len = page_dates.len();
        dates.extend(page_dates);
        let count = value
            .get("count")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(offset + page_len);
        offset += page_len;
        if page_len == 0 || offset >= count {
            break;
        }
    }
    Ok(dates)
}

pub fn collect_fred(
    config: &Config,
    db: &Db,
    start: &str,
    initial_release: bool,
    series_filter: Option<&str>,
) -> Result<CollectionReport, Box<dyn Error>> {
    let key = config.fred_api_key.as_ref().ok_or("FRED_API_KEY missing")?;
    if let Some(series) = series_filter {
        if !FRED_SERIES.contains(&series) {
            return Err(format!("unknown FRED series: {series}").into());
        }
    }
    let http = client(config)?;
    let mut report = CollectionReport::default();

    for series in FRED_SERIES
        .iter()
        .copied()
        .filter(|series| series_filter.is_none_or(|filter| *series == filter))
    {
        let windows = if initial_release {
            match fred_vintage_dates(&http, key, series, start) {
                Ok(dates) => fred_vintage_windows(&dates),
                Err(error) => {
                    report.error(series, error);
                    continue;
                }
            }
        } else {
            Vec::new()
        };
        let request_windows = if initial_release {
            windows
        } else {
            vec![(String::new(), String::new())]
        };

        for (realtime_start, realtime_end) in request_windows {
            let mut query = vec![
                ("series_id", series.to_string()),
                ("api_key", key.to_string()),
                ("file_type", "json".to_string()),
                ("observation_start", start.to_string()),
                ("limit", "100000".to_string()),
            ];
            if initial_release {
                query.extend([
                    ("output_type", "4".to_string()),
                    ("realtime_start", realtime_start),
                    ("realtime_end", realtime_end),
                ]);
            }
            let value = match fred_json(
                &http,
                "https://api.stlouisfed.org/fred/series/observations",
                &query,
                key,
            ) {
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
                let released_at = initial_release.then(|| {
                    observation
                        .get("realtime_start")
                        .and_then(Value::as_str)
                        .map(date_to_end_of_day)
                });
                let released_at = released_at.flatten();
                let source_asof = observation
                    .get("realtime_end")
                    .and_then(Value::as_str)
                    .map(date_to_end_of_day);
                let revision_id = fred_revision_id(initial_release, released_at.as_deref(), number);
                report.record(db.put(&NewObservation {
                    source: if initial_release { "alfred" } else { "fred" }.into(),
                    series: series.into(),
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

#[derive(Clone, Copy)]
enum KrxKind {
    Index(&'static str),
    Breadth(&'static str),
    Futures,
    Options,
}

struct KrxService {
    name: &'static str,
    api_id: &'static str,
    path: &'static str,
    kind: KrxKind,
}

const KRX_SERVICES: &[KrxService] = &[
    KrxService {
        name: "KOSPI 시리즈 일별시세정보",
        api_id: "kospi_dd_trd",
        path: "idx/kospi_dd_trd",
        kind: KrxKind::Index("KOSPI"),
    },
    KrxService {
        name: "KOSDAQ 시리즈 일별시세정보",
        api_id: "kosdaq_dd_trd",
        path: "idx/kosdaq_dd_trd",
        kind: KrxKind::Index("KOSDAQ"),
    },
    KrxService {
        name: "유가증권 일별매매정보",
        api_id: "stk_bydd_trd",
        path: "sto/stk_bydd_trd",
        kind: KrxKind::Breadth("KOSPI"),
    },
    KrxService {
        name: "코스닥 일별매매정보",
        api_id: "ksq_bydd_trd",
        path: "sto/ksq_bydd_trd",
        kind: KrxKind::Breadth("KOSDAQ"),
    },
    KrxService {
        name: "선물 일별매매정보 (주식선물外)",
        api_id: "fut_bydd_trd",
        path: "drv/fut_bydd_trd",
        kind: KrxKind::Futures,
    },
    KrxService {
        name: "옵션 일별매매정보 (주식옵션外)",
        api_id: "opt_bydd_trd",
        path: "drv/opt_bydd_trd",
        kind: KrxKind::Options,
    },
];

pub fn collect_krx(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let key = config.krx_api_key.as_ref().ok_or("KRX_API_KEY missing")?;
    let http = client(config)?;
    let mut report = CollectionReport::default();
    let today = Utc::now().date_naive();
    let dates = (2..=config.krx_lookback_days + 2)
        .rev()
        .map(|offset| today - ChronoDuration::days(offset as i64))
        .filter(|date| !matches!(date.weekday(), Weekday::Sat | Weekday::Sun))
        .collect::<Vec<_>>();

    for service in KRX_SERVICES {
        let mut recognized = 0usize;
        for date in &dates {
            let response = match http
                .get(format!(
                    "https://data-dbg.krx.co.kr/svc/apis/{}",
                    service.path
                ))
                .header("AUTH_KEY", key)
                .query(&[("basDd", date.format("%Y%m%d").to_string())])
                .send()
            {
                Ok(response) => response,
                Err(error) => {
                    report.error(service.api_id, error);
                    break;
                }
            };
            let status = response.status();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                report.error(
                    service.api_id,
                    format!(
                        "KRX service not authorized; apply for '{}' in KRX Open API (HTTP {})",
                        service.name, status
                    ),
                );
                break;
            }
            if !status.is_success() {
                report.error(service.api_id, format!("HTTP {status}"));
                if status.as_u16() == 429 {
                    break;
                }
                continue;
            }
            let value: Value = match response.json() {
                Ok(value) => value,
                Err(error) => {
                    report.error(service.api_id, error);
                    continue;
                }
            };
            let Some(rows) = value
                .get("OutBlock_1")
                .and_then(Value::as_array)
                .or_else(|| find_object_rows(&value))
            else {
                continue;
            };
            recognized += store_krx_service(db, &mut report, service.kind, *date, rows);
        }
        if recognized == 0
            && !report
                .errors
                .iter()
                .any(|error| error.starts_with(service.api_id))
        {
            report.error(
                service.api_id,
                "no recognized rows in the requested lookback",
            );
        }
    }
    Ok(report)
}

fn store_krx_service(
    db: &Db,
    report: &mut CollectionReport,
    kind: KrxKind,
    date: NaiveDate,
    rows: &[Value],
) -> usize {
    match kind {
        KrxKind::Index(market) => store_krx_index(db, report, market, date, rows),
        KrxKind::Breadth(market) => store_krx_breadth(db, report, market, date, rows),
        KrxKind::Futures => store_krx_futures(db, report, date, rows),
        KrxKind::Options => store_krx_options(db, report, date, rows),
    }
}

fn store_krx_index(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    date: NaiveDate,
    rows: &[Value],
) -> usize {
    let target = if market == "KOSPI" {
        "코스피"
    } else {
        "코스닥"
    };
    let Some(row) = rows.iter().find(|row| {
        row.get("IDX_NM")
            .and_then(Value::as_str)
            .is_some_and(|name| name.trim().eq_ignore_ascii_case(target) || name.trim() == market)
    }) else {
        return 0;
    };
    let mut stored = 0;
    if let Some(value) = parse_number(row.get("CLSPRC_IDX")) {
        store_krx_value(
            db,
            report,
            &format!("KRX_{market}_CLOSE"),
            date,
            value,
            json!({"index":target}),
        );
        stored += 1;
    }
    if let Some(value) = parse_number(row.get("FLUC_RT")) {
        store_krx_value(
            db,
            report,
            &format!("KRX_{market}_RETURN"),
            date,
            value,
            json!({"index":target}),
        );
        stored += 1;
    }
    stored
}

fn store_krx_breadth(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    date: NaiveDate,
    rows: &[Value],
) -> usize {
    let changes = rows
        .iter()
        .filter_map(|row| {
            parse_number(row.get("FLUC_RT")).or_else(|| parse_number(row.get("CMPPREVDD_PRC")))
        })
        .collect::<Vec<_>>();
    let advances = changes.iter().filter(|value| **value > 0.0).count();
    let declines = changes.iter().filter(|value| **value < 0.0).count();
    if advances + declines == 0 {
        return 0;
    }
    let breadth = 100.0 * advances as f64 / (advances + declines) as f64;
    store_krx_value(
        db,
        report,
        &format!("KRX_{market}_BREADTH"),
        date,
        breadth,
        json!({"advances":advances,"declines":declines}),
    );
    1
}

fn store_krx_futures(
    db: &Db,
    report: &mut CollectionReport,
    date: NaiveDate,
    rows: &[Value],
) -> usize {
    let open_interest = rows
        .iter()
        .filter_map(|row| {
            parse_number(row.get("OPNINT_QTY")).or_else(|| parse_number(row.get("OPEN_INT")))
        })
        .sum::<f64>();
    if open_interest <= 0.0 {
        return 0;
    }
    store_krx_value(
        db,
        report,
        "KRX_FUTURES_OI",
        date,
        open_interest,
        json!({"contracts":rows.len()}),
    );
    1
}

fn store_krx_options(
    db: &Db,
    report: &mut CollectionReport,
    date: NaiveDate,
    rows: &[Value],
) -> usize {
    let mut put_volume = 0.0;
    let mut call_volume = 0.0;
    for row in rows {
        let label = ["RGHT_TP_NM", "PUT_CALL_TP_NM", "ISU_NM"]
            .iter()
            .find_map(|field| row.get(*field).and_then(Value::as_str))
            .unwrap_or("")
            .to_uppercase();
        let volume = ["ACC_TRDVOL", "TRD_VOL"]
            .iter()
            .find_map(|field| parse_number(row.get(*field)))
            .unwrap_or(0.0);
        let tokens = label.split_whitespace().collect::<Vec<_>>();
        if label.contains("풋") || label.contains("PUT") || tokens.contains(&"P") {
            put_volume += volume;
        } else if label.contains("콜") || label.contains("CALL") || tokens.contains(&"C") {
            call_volume += volume;
        }
    }
    if call_volume <= f64::EPSILON {
        return 0;
    }
    let ratio = put_volume / call_volume;
    store_krx_value(
        db,
        report,
        "KRX_PUT_CALL_RATIO",
        date,
        ratio,
        json!({"put_volume":put_volume,"call_volume":call_volume}),
    );
    1
}

fn store_krx_value(
    db: &Db,
    report: &mut CollectionReport,
    series: &str,
    date: NaiveDate,
    value: f64,
    metadata: Value,
) {
    let observed_at = date.to_string();
    let released_at = (date + ChronoDuration::days(2))
        .and_hms_opt(23, 59, 59)
        .map(|value| value.and_utc().to_rfc3339());
    report.record(db.put(&NewObservation {
        source: "krx".into(),
        series: series.into(),
        entity: String::new(),
        observed_at,
        value,
        released_at,
        source_asof: Some(Utc::now().to_rfc3339()),
        revision_id: Some(format!("value:{value:.17}")),
        metadata,
    }));
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

    #[test]
    fn fred_vintage_windows_stay_below_api_limit_without_overlap() {
        let dates = (0..FRED_VINTAGE_WINDOW_SIZE + 1)
            .map(|index| format!("vintage-{index:04}"))
            .collect::<Vec<_>>();
        let windows = fred_vintage_windows(&dates);
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0], ("vintage-0000".into(), "vintage-0099".into()));
        assert_eq!(windows[1], ("vintage-0100".into(), "vintage-0100".into()));
    }

    #[test]
    fn fred_current_and_alfred_initial_revisions_have_distinct_ids() {
        assert_eq!(
            fred_revision_id(false, Some("2026-08-20T23:59:59Z"), 1.25),
            "current:1.25000000000000000"
        );
        assert_eq!(
            fred_revision_id(true, Some("2020-01-02T23:59:59Z"), 1.25),
            "2020-01-02T23:59:59Z"
        );
    }

    #[test]
    fn krx_stock_rows_produce_market_breadth() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("krx.db")).unwrap();
        let mut report = CollectionReport::default();
        let rows = vec![
            json!({"FLUC_RT":"1.2"}),
            json!({"FLUC_RT":"-0.4"}),
            json!({"FLUC_RT":"0.0"}),
        ];
        let date = NaiveDate::from_ymd_opt(2026, 8, 18).unwrap();
        assert_eq!(store_krx_breadth(&db, &mut report, "KOSPI", date, &rows), 1);
        let point = db
            .latest("krx", "KRX_KOSPI_BREADTH", None)
            .unwrap()
            .unwrap();
        assert_eq!(point.value, 50.0);
    }

    #[test]
    fn krx_option_rows_produce_put_call_ratio() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("krx.db")).unwrap();
        let mut report = CollectionReport::default();
        let rows = vec![
            json!({"RGHT_TP_NM":"풋","ACC_TRDVOL":"300"}),
            json!({"RGHT_TP_NM":"콜","ACC_TRDVOL":"200"}),
        ];
        let date = NaiveDate::from_ymd_opt(2026, 8, 18).unwrap();
        assert_eq!(store_krx_options(&db, &mut report, date, &rows), 1);
        let point = db
            .latest("krx", "KRX_PUT_CALL_RATIO", None)
            .unwrap()
            .unwrap();
        assert_eq!(point.value, 1.5);
    }
}
