mod collectors;
mod config;
mod db;
mod dsl;
mod engine;
mod rulebook;
mod scoring;
mod server;

use collectors::CollectionReport;
use config::Config;
use db::{Db, NewObservation};
use std::{error::Error, io};

fn usage() {
    println!(
        "EconomicsRadar 0.3.4\n\
         commands:\n\
           keys\n\
           rulebook\n\
           collect-fred [start] [series]\n\
           collect-alfred [start] [series]\n\
           collect-public\n\
           collect-ecos [series]\n\
           collect-krx\n\
           collect-official\n\
           collect-all [start]\n\
           run [as-of]\n\
           backtest <start> <end> [max-points]\n\
           serve\n\
           demo"
    );
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
        match collectors::collect_krx(config, db) {
            Ok(result) => report.merge(result),
            Err(error) => report.errors.push(format!("krx: {error}")),
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

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load();
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    match command.as_str() {
        "keys" => config.print_key_status(),
        "rulebook" => {
            let verification = rulebook::verify(&config.rulebook_path)?;
            println!("{}", serde_json::to_string_pretty(&verification)?);
        }
        "collect-fred" | "collect-alfred" => {
            let db = Db::open(&config.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            let series = std::env::args().nth(3);
            let report = collectors::collect_fred(
                &config,
                &db,
                &start,
                command == "collect-alfred",
                series.as_deref(),
            )?;
            print_report(&report)?;
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
            print_report(&collectors::collect_krx(&config, &db)?)?;
        }
        "collect-all" => {
            let db = Db::open(&config.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            let mut report = collectors::collect_fred(&config, &db, &start, false, None)?;
            report.merge(collectors::collect_fred(&config, &db, &start, true, None)?);
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
            server::serve(&config.host, config.db_path.clone())?;
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
