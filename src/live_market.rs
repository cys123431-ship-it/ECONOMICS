use crate::{
    collectors::CollectionReport,
    config::Config,
    db::{Db, NewObservation},
    krx_analytics,
};
use chrono::{Datelike, FixedOffset, NaiveDate, SecondsFormat, Utc, Weekday};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::{error::Error, time::Duration};

#[derive(Clone, Copy)]
enum FastKrxKind {
    Index(&'static str),
    Breadth(&'static str),
    Futures,
    Options,
}

#[derive(Clone, Copy)]
struct FastKrxService {
    api_id: &'static str,
    path: &'static str,
    kind: FastKrxKind,
    always_poll: bool,
}

const FAST_KRX_SERVICES: &[FastKrxService] = &[
    FastKrxService {
        api_id: "kospi_dd_trd",
        path: "idx/kospi_dd_trd",
        kind: FastKrxKind::Index("KOSPI"),
        always_poll: true,
    },
    FastKrxService {
        api_id: "kosdaq_dd_trd",
        path: "idx/kosdaq_dd_trd",
        kind: FastKrxKind::Index("KOSDAQ"),
        always_poll: true,
    },
    FastKrxService {
        api_id: "stk_bydd_trd",
        path: "sto/stk_bydd_trd",
        kind: FastKrxKind::Breadth("KOSPI"),
        always_poll: false,
    },
    FastKrxService {
        api_id: "ksq_bydd_trd",
        path: "sto/ksq_bydd_trd",
        kind: FastKrxKind::Breadth("KOSDAQ"),
        always_poll: false,
    },
    FastKrxService {
        api_id: "fut_bydd_trd",
        path: "drv/fut_bydd_trd",
        kind: FastKrxKind::Futures,
        always_poll: false,
    },
    FastKrxService {
        api_id: "opt_bydd_trd",
        path: "drv/opt_bydd_trd",
        kind: FastKrxKind::Options,
        always_poll: false,
    },
];

fn http_client(config: &Config) -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(config.http_timeout_secs.min(20)))
        .user_agent(concat!("ECONOMICS-Radar/", env!("CARGO_PKG_VERSION")))
        .build()?)
}

fn korea_today() -> NaiveDate {
    let kst = FixedOffset::east_opt(9 * 60 * 60).expect("KST offset is valid");
    Utc::now().with_timezone(&kst).date_naive()
}

fn recent_business_dates(today: NaiveDate, count: usize) -> Vec<NaiveDate> {
    let mut dates = Vec::with_capacity(count);
    let mut date = today;
    while dates.len() < count {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            dates.push(date);
        }
        date = date.pred_opt().expect("modern dates have predecessors");
    }
    dates
}

fn parse_number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.replace(',', "").trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn rows(value: &Value) -> Option<&Vec<Value>> {
    value
        .get("OutBlock_1")
        .and_then(Value::as_array)
        .or_else(|| value.as_object()?.values().find_map(Value::as_array))
}

fn store(
    db: &Db,
    report: &mut CollectionReport,
    series: &str,
    date: NaiveDate,
    value: f64,
    metadata: Value,
) {
    report.attempted += 1;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    match db.put(&NewObservation {
        source: "krx".into(),
        series: series.into(),
        entity: String::new(),
        observed_at: date.to_string(),
        value,
        released_at: Some(now.clone()),
        source_asof: Some(now),
        revision_id: Some(format!("fast:{date}:{value:.17}")),
        metadata,
    }) {
        Ok(true) => report.stored += 1,
        Ok(false) => report.unchanged += 1,
        Err(error) => report.errors.push(format!("{series}: database: {error}")),
    }
    if let Err(error) = db.mark_series_checked("krx", series, &date.to_string()) {
        report
            .errors
            .push(format!("{series}: series status: {error}"));
    }
}

fn fetch_rows(
    http: &Client,
    key: &str,
    service: FastKrxService,
    date: NaiveDate,
) -> Result<Option<Value>, String> {
    let response = http
        .get(format!(
            "https://data-dbg.krx.co.kr/svc/apis/{}",
            service.path
        ))
        .header("AUTH_KEY", key)
        .query(&[("basDd", date.format("%Y%m%d").to_string())])
        .send()
        .map_err(|error| format!("{}: {error}", service.api_id))?;
    let status = response.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(format!(
            "{}: service is not authorized (HTTP {status})",
            service.api_id
        ));
    }
    if status.as_u16() == 429 {
        return Err(format!("{}: rate limited (HTTP 429)", service.api_id));
    }
    if !status.is_success() {
        return Err(format!("{}: HTTP {status}", service.api_id));
    }
    let value: Value = response
        .json()
        .map_err(|error| format!("{}: invalid JSON: {error}", service.api_id))?;
    Ok(rows(&value)
        .is_some_and(|rows| !rows.is_empty())
        .then_some(value))
}

fn store_index(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    date: NaiveDate,
    rows: &[Value],
) {
    let target = if market == "KOSPI" {
        "코스피"
    } else {
        "코스닥"
    };
    let Some(row) = rows.iter().find(|row| {
        row.get("IDX_NM")
            .and_then(Value::as_str)
            .is_some_and(|name| name.trim() == target || name.trim().eq_ignore_ascii_case(market))
    }) else {
        return;
    };
    if let Some(value) = parse_number(row.get("CLSPRC_IDX")) {
        store(
            db,
            report,
            &format!("KRX_{market}_CLOSE"),
            date,
            value,
            json!({"fast_refresh":true,"index":target}),
        );
    }
    if let Some(value) = parse_number(row.get("FLUC_RT")) {
        store(
            db,
            report,
            &format!("KRX_{market}_RETURN"),
            date,
            value,
            json!({"fast_refresh":true,"index":target}),
        );
    }
}

fn store_breadth(
    db: &Db,
    report: &mut CollectionReport,
    market: &str,
    date: NaiveDate,
    rows: &[Value],
) {
    let changes = rows
        .iter()
        .filter_map(|row| {
            parse_number(row.get("FLUC_RT")).or_else(|| parse_number(row.get("CMPPREVDD_PRC")))
        })
        .collect::<Vec<_>>();
    let advances = changes.iter().filter(|value| **value > 0.0).count();
    let declines = changes.iter().filter(|value| **value < 0.0).count();
    if advances + declines == 0 {
        return;
    }
    let value = 100.0 * advances as f64 / (advances + declines) as f64;
    store(
        db,
        report,
        &format!("KRX_{market}_BREADTH"),
        date,
        value,
        json!({"fast_refresh":true,"advances":advances,"declines":declines}),
    );
    store(
        db,
        report,
        &format!("KRX_{market}_BREADTH_COUNT"),
        date,
        (advances + declines) as f64,
        json!({"fast_refresh":true,"advances":advances,"declines":declines}),
    );
}

fn store_futures(db: &Db, report: &mut CollectionReport, date: NaiveDate, rows: &[Value]) {
    let stats = krx_analytics::futures_stats(rows);
    if stats.regular_open_interest > 0.0 {
        store(
            db,
            report,
            "KRX_FUTURES_OI",
            date,
            stats.regular_open_interest,
            json!({
                "fast_refresh":true,
                "product":"코스피200 선물",
                "session":"정규",
                "scope":"outright-all-maturities",
                "contracts":stats.regular_contracts
            }),
        );
    }
    if let Some(value) = stats.front_month_basis {
        store(
            db,
            report,
            "KRX_BASIS",
            date,
            value,
            json!({
                "fast_refresh":true,
                "product":"코스피200 선물",
                "session":"정규",
                "contract":stats.front_contract,
                "method":"front-month-settlement-minus-spot"
            }),
        );
    }
}

fn store_options(db: &Db, report: &mut CollectionReport, date: NaiveDate, rows: &[Value]) {
    let stats = krx_analytics::options_stats(rows);
    if let Some(value) = stats.put_call_ratio {
        store(
            db,
            report,
            "KRX_PUT_CALL",
            date,
            value,
            json!({
                "fast_refresh":true,
                "put_volume":stats.put_volume,
                "call_volume":stats.call_volume,
                "session":"정규",
                "expiry":stats.expiry
            }),
        );
    }
    if let Some(value) = stats.active_implied_volatility {
        store(
            db,
            report,
            "KRX_OPTIONS_AVG_IMPLIED_VOL",
            date,
            value,
            json!({
                "fast_refresh":true,
                "active_contracts":stats.active_contracts,
                "session":"정규",
                "expiry":stats.expiry,
                "method":"front-month-positive-volume-weighted"
            }),
        );
    }
}

fn store_combined_breadth(db: &Db, report: &mut CollectionReport) {
    let Ok(Some(kospi)) = db.latest("krx", "KRX_KOSPI_BREADTH", None) else {
        return;
    };
    let Ok(Some(kosdaq)) = db.latest("krx", "KRX_KOSDAQ_BREADTH", None) else {
        return;
    };
    if kospi.observed_at != kosdaq.observed_at {
        return;
    }
    let Ok(Some(kospi_count)) = db.latest("krx", "KRX_KOSPI_BREADTH_COUNT", None) else {
        return;
    };
    let Ok(Some(kosdaq_count)) = db.latest("krx", "KRX_KOSDAQ_BREADTH_COUNT", None) else {
        return;
    };
    if kospi_count.observed_at != kospi.observed_at
        || kosdaq_count.observed_at != kospi.observed_at
        || kospi_count.value + kosdaq_count.value <= f64::EPSILON
    {
        return;
    }
    let Ok(date) = NaiveDate::parse_from_str(
        &kospi.observed_at[..10.min(kospi.observed_at.len())],
        "%Y-%m-%d",
    ) else {
        return;
    };
    store(
        db,
        report,
        "KRX_BREADTH",
        date,
        (kospi.value * kospi_count.value + kosdaq.value * kosdaq_count.value)
            / (kospi_count.value + kosdaq_count.value),
        json!({
            "fast_refresh":true,
            "components":["KOSPI","KOSDAQ"],
            "kospi_issues":kospi_count.value,
            "kosdaq_issues":kosdaq_count.value,
            "method":"issue-count-weighted"
        }),
    );
}

pub fn collect_krx_fast(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let Some(key) = config.krx_api_key.as_deref() else {
        return Ok(CollectionReport::default());
    };
    let http = http_client(config)?;
    let today = korea_today();
    let dates = recent_business_dates(today, 5);
    let mut report = CollectionReport::default();

    for service in FAST_KRX_SERVICES {
        let mut found = false;
        for date in &dates {
            match fetch_rows(&http, key, *service, *date) {
                Ok(Some(value)) => {
                    if let Some(rows) = rows(&value) {
                        match service.kind {
                            FastKrxKind::Index(market) => {
                                store_index(db, &mut report, market, *date, rows)
                            }
                            FastKrxKind::Breadth(market) => {
                                store_breadth(db, &mut report, market, *date, rows)
                            }
                            FastKrxKind::Futures => store_futures(db, &mut report, *date, rows),
                            FastKrxKind::Options => store_options(db, &mut report, *date, rows),
                        }
                    }
                    found = true;
                    break;
                }
                Ok(None) => continue,
                Err(error) => {
                    report.errors.push(error);
                    break;
                }
            }
        }
        if !found
            && service.always_poll
            && report.errors.iter().all(|e| !e.starts_with(service.api_id))
        {
            report.errors.push(format!(
                "{}: no published rows in the latest five business dates",
                service.api_id
            ));
        }
    }
    store_combined_breadth(db, &mut report);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_business_dates_include_today_and_are_descending() {
        let friday = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let dates = recent_business_dates(friday, 4);
        assert_eq!(dates[0], friday);
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2026, 8, 20).unwrap());
        assert_eq!(dates[3], NaiveDate::from_ymd_opt(2026, 8, 18).unwrap());
    }

    #[test]
    fn commas_are_accepted_in_numeric_fields() {
        assert_eq!(parse_number(Some(&json!("6,900.25"))), Some(6900.25));
    }
}
