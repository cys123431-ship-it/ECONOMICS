mod collectors;
mod config;
mod db;
mod dsl;
mod engine;
mod rulebook;
mod scoring;
mod server;
use config::Config;
use db::Db;
use std::error::Error;

fn usage() {
    println!("EconomicsRadar 0.2.0\ncommands: keys | rulebook | collect-fred [start] | collect-alfred [start] | collect-official | run | serve | demo");
}
fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::load();
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    match cmd.as_str() {
        "keys" => cfg.print_key_status(),
        "rulebook" => rulebook::verify()?,
        "collect-fred" => {
            let db = Db::open(&cfg.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            println!(
                "stored={}",
                collectors::collect_fred(&cfg, &db, &start, false)?
            );
        }
        "collect-alfred" => {
            let db = Db::open(&cfg.db_path)?;
            let start = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "2000-01-01".into());
            println!(
                "stored={}",
                collectors::collect_fred(&cfg, &db, &start, true)?
            );
        }
        "collect-official" => {
            let db = Db::open(&cfg.db_path)?;
            let mut n = 0;
            n += collectors::collect_treasury(&db).unwrap_or_else(|e| {
                eprintln!("treasury: {e}");
                0
            });
            n += collectors::collect_binance(&cfg, &db).unwrap_or_else(|e| {
                eprintln!("binance: {e}");
                0
            });
            n += collectors::collect_ecos(&cfg, &db).unwrap_or_else(|e| {
                eprintln!("ecos: {e}");
                0
            });
            n += collectors::collect_krx(&cfg, &db).unwrap_or_else(|e| {
                eprintln!("krx: {e}");
                0
            });
            println!("stored={n}");
        }
        "run" => {
            let db = Db::open(&cfg.db_path)?;
            println!("{}", serde_json::to_string_pretty(&engine::run(&db)?)?);
        }
        "serve" => server::serve(&cfg.host, cfg.db_path.clone())?,
        "demo" => {
            let db = Db::open(&cfg.db_path)?;
            for i in 0..40 {
                db.put(
                    "fred",
                    "VIXCLS",
                    &format!("2026-07-{i:02}"),
                    15.0 + i as f64,
                    None,
                )?;
            }
            db.put("fred", "VIXCLS", "2026-08-20", 45.0, None)?;
            println!("{}", serde_json::to_string_pretty(&engine::run(&db)?)?);
        }
        _ => usage(),
    }
    Ok(())
}
