mod collectors;
mod config;
mod dashboard;
mod db;
mod dsl;
mod engine;
mod live_market;
mod refresh;
mod rulebook;
mod scoring;
mod server;

use collectors::CollectionReport;
use config::Config;
use db::{Db, NewObservation};
use std::{error::Error, io, process::Command, time::Duration};

const ALFRED_SKIP_SERIES: &[&str] = &["SP500", "DJIA", "EQTA"];
const ALFRED_SERIES: &[&str] = &[
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
    "NASDAQCOM",
    "DGS10",
    "DGS2",
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
    "KORLOLITOAASTSAM",
    "CHNLOLITOAASTSAM",
];

fn usage() {
    println!(
        "EconomicsRadar {}\n\
         commands:\n\
           launch\n\
           keys\n\
           rulebook\n\
           collect-fred [start] [series]\n\
           collect-alfred [start] [series]\n\
           collect-public\n\
           collect-ecos [series]\n\
           collect-krx [api-id]\n\
           collect-krx-live\n\
           collect-official\n\
           collect-all [start]\n\
           run [as-of]\n\
           backtest <start> <end> [max-points]\n\
           serve\n\
           demo",
        env!("CARGO_PKG_VERSION")
    );
}

fn dashboard_url(host: &str) -> String {
    let browser_host = host
        .strip_prefix("0.0.0.0:")
        .map(|port| format!("127.0.0.1:{port}"))
        .or_else(|| {
            host.strip_prefix("[::]:")
                .map(|port| format!("127.0.0.1:{port}"))
        })
        .unwrap_or_else(|| host.to_string());
    format!("http://{browser_host}/")
}

fn dashboard_is_running(url: &str) -> bool {
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
    else {
        return false;
    };
    client
        .get(format!("{url}health"))
        .send()
        .ok()
        .filter(|response| response.status().is_success())
        .and_then(|response| response.json::<serde_json::Value>().ok())
        .and_then(|body| {
            body.get("status")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .is_some_and(|status| status == "ok")
}

#[cfg(target_os = "windows")]
fn open_dashboard(url: &str) -> io::Result<()> {
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_dashboard(url: &str) -> io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_dashboard(url: &str) -> io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

fn print_report(report: &CollectionReport) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_json::to_string_pretty(report)?);
    if report.is_clean() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "collection completed with {} error(s)",
            report.errors.len()
        ))
        .into())
    }
}

fn collect_public(config: &Config, db: &Db) -> CollectionReport {
    let mut report = CollectionReport::default();
    match collectors::collect_treasury(config, db) {
        Ok(result) => report.merge(result),
        Err(error) => report.errors.push(format!("treasury: {error}")),
    }
    match collectors::collect_binance(config, db) {
        Ok(result) => report.merge(result),
        Err(error) => report.errors.push(format!("binance: {error}")),
    }
    report
}

fn collect_official(config: &Config, db: &Db) -> Result<CollectionReport, Box<dyn Error>> {
    let mut report = collect_public(config, db);
    if config.ecos_api_key.is_some() {
        match collectors::collect_ecos(config, db, None) {
            Ok(result) => report.merge(result),
            Err(error) => report.errors.push(format!("ecos: {error}")),
        }
    }
    if config.krx_api_key.is_some() {
        match collectors::collect_krx(config, db, None) {
            Ok(result) => report.merge(result),
            Err(error) => report.errors.push(format!("krx: {error}")),
        }
        match live_market::collect_krx_fast(config, db) {
            Ok(result) => report.merge(result),
            Err(error) => report.errors.push(format!("krx latest: {error}")),
        }
    }
    match collectors::collect_configured_adapters(config, db) {
        Ok(result) => report.merge(result),
        Err(error) => report
            .errors
            .push(format!("configured official adapters: {error}")),
    }
    Ok(report)
}

fn collect_alfred_safe(
    config: &Config,
    db: &Db,
    start: &str,
    series: Option<&str>,
) -> Result<CollectionReport, Box<dyn Error>> {
    if let Some(series) = series {
        if ALFRED_SKIP_SERIES.contains(&series) {
            eprintln!(
                "ALFRED vintage collection skipped for {series}: FRED vintage history is unavailable or unsuitable for automatic polling; current FRED observations remain enabled"
            );
            return Ok(CollectionReport::default());
        }
        return collectors::collect_fred(config, db, start, true, Some(series));
    }

    let mut report = CollectionReport::default();
    for series in ALFRED_SERIES {
        match collectors::collect_fred(config, db, start, true, Some(series)) {
            Ok(collected) => report.merge(collected),
            Err(error) => report.errors.push(format!("{series}: {error}")),
        }
    }
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load();
    let command = std::env::args().nth(1).unwrap_or_else(|| "launch".into());
    match command.as_str() {
        "launch" => {
            rulebook::verify(&config.rulebook_path)?;
            let url = dashboard_url(&config.host);
            if dashboard_is_running(&url) {
                open_dashboard(&url)?;
            } else {
                let browser_url = url.clone();
                let refresh = refresh::start(config.clone());
                server::serve_with_refresh(
                    &config.host,
                    config.db_path.clone(),
                    refresh,
                    move || {
                        if let Err(error) = open_dashboard(&browser_url) {
                            eprintln!("could not open dashboard browser: {error}");
                        }
                    },
                )?;
            }
        }
        "keys" => config.print_key_status(),
        "rulebook" => {
            let verification = rulebook::verify(&config.rulebook_path)?;
            println!("{}", serde_json::to_string_pretty(&verification)?);
        }
        "collect-fred" => {
            let db = Db::open(&config.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            let series = std::env::args().nth(3);
            print_report(&collectors::collect_fred(
                &config,
                &db,
                &start,
                false,
                series.as_deref(),
            )?)?;
        }
        "collect-alfred" => {
            let db = Db::open(&config.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            let series = std::env::args().nth(3);
            print_report(&collect_alfred_safe(
                &config,
                &db,
                &start,
                series.as_deref(),
            )?)?;
        }
        "collect-official" => {
            let db = Db::open(&config.db_path)?;
            let report = collect_official(&config, &db)?;
            print_report(&report)?;
        }
        "collect-public" => {
            let db = Db::open(&config.db_path)?;
            print_report(&collect_public(&config, &db))?;
        }
        "collect-ecos" => {
            let db = Db::open(&config.db_path)?;
            let series = std::env::args().nth(2);
            print_report(&collectors::collect_ecos(&config, &db, series.as_deref())?)?;
        }
        "collect-krx" => {
            let db = Db::open(&config.db_path)?;
            let service = std::env::args().nth(2);
            print_report(&collectors::collect_krx(&config, &db, service.as_deref())?)?;
        }
        "collect-krx-live" => {
            let db = Db::open(&config.db_path)?;
            print_report(&live_market::collect_krx_fast(&config, &db)?)?;
        }
        "collect-all" => {
            let db = Db::open(&config.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            let mut report = collectors::collect_fred(&config, &db, &start, false, None)?;
            report.merge(collect_official(&config, &db)?);
            print_report(&report)?;
        }
        "run" => {
            rulebook::verify(&config.rulebook_path)?;
            let db = Db::open(&config.db_path)?;
            let snapshot = if let Some(as_of) = std::env::args().nth(2) {
                engine::run_at(&db, &config, &as_of, true)?
            } else {
                engine::run(&db, &config)?
            };
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        }
        "backtest" => {
            rulebook::verify(&config.rulebook_path)?;
            let start = std::env::args().nth(2).ok_or("backtest start missing")?;
            let end = std::env::args().nth(3).ok_or("backtest end missing")?;
            let max_points = std::env::args()
                .nth(4)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(5_000);
            let db = Db::open(&config.db_path)?;
            let dates = db.observation_dates(&start, &end)?;
            if dates.len() > max_points {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "backtest has {} dates, above max-points={max_points}; pass a larger explicit limit",
                        dates.len()
                    ),
                )
                .into());
            }
            let mut snapshots = Vec::with_capacity(dates.len());
            for date in dates {
                snapshots.push(engine::run_at(
                    &db,
                    &config,
                    &format!("{date}T23:59:59Z"),
                    true,
                )?);
            }
            println!("{}", serde_json::to_string_pretty(&snapshots)?);
        }
        "serve" => {
            rulebook::verify(&config.rulebook_path)?;
            let refresh = refresh::start(config.clone());
            server::serve_with_refresh(&config.host, config.db_path.clone(), refresh, || {})?;
        }
        "demo" => {
            rulebook::verify(&config.rulebook_path)?;
            let db = Db::open(&config.db_path)?;
            let start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
            for offset in 0..90 {
                let date = start + chrono::Duration::days(offset);
                let value = 15.0 + ((offset as f64 / 7.0).sin() * 3.0);
                db.put(&NewObservation::simple(
                    "fred",
                    "VIXCLS",
                    &date.to_string(),
                    value,
                ))?;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&engine::run(&db, &config)?)?
            );
        }
        "help" | "--help" | "-h" => usage(),
        _ => {
            usage();
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown command: {command}"),
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_url_uses_loopback_for_wildcard_bindings() {
        assert_eq!(dashboard_url("127.0.0.1:8765"), "http://127.0.0.1:8765/");
        assert_eq!(dashboard_url("0.0.0.0:9000"), "http://127.0.0.1:9000/");
        assert_eq!(dashboard_url("[::]:7000"), "http://127.0.0.1:7000/");
    }

    #[test]
    fn auto_alfred_skip_list_contains_problematic_series() {
        assert!(ALFRED_SKIP_SERIES.contains(&"SP500"));
        assert!(ALFRED_SKIP_SERIES.contains(&"DJIA"));
        assert!(ALFRED_SKIP_SERIES.contains(&"EQTA"));
    }
}
