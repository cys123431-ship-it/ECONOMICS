use crate::{collectors, config::Config, db::Db, engine, live_market};
use chrono::{Duration as ChronoDuration, SecondsFormat, Utc};
use serde::Serialize;
use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const INCREMENTAL_LOOKBACK_DAYS: i64 = 120;
const FRED_CURRENT_SERIES: &[&str] = &[
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
    "SP500",
    "NASDAQCOM",
    "DJIA",
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

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum RefreshMode {
    Fast,
    Full,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RefreshStatus {
    pub running: bool,
    pub queued: bool,
    mode: Option<RefreshMode>,
    pub phase: Option<String>,
    pub last_started_at: Option<String>,
    pub last_completed_at: Option<String>,
    pub last_snapshot_as_of: Option<String>,
    pub attempted: usize,
    pub stored: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct RefreshControl {
    sender: SyncSender<RefreshMode>,
    status: Arc<Mutex<RefreshStatus>>,
}

impl RefreshControl {
    pub fn request_full(&self) -> bool {
        with_status(&self.status, |status| status.queued = true);
        match self.sender.try_send(RefreshMode::Full) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                with_status(&self.status, |status| status.queued = false);
                false
            }
        }
    }

    pub fn status_json(&self) -> String {
        let status = self
            .status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| RefreshStatus {
                errors: vec!["refresh status lock is unavailable".into()],
                ..RefreshStatus::default()
            });
        serde_json::to_string(&status).unwrap_or_else(|_| r#"{"running":false}"#.into())
    }
}

pub fn start(config: Config) -> RefreshControl {
    let (sender, receiver) = mpsc::sync_channel(1);
    let status = Arc::new(Mutex::new(RefreshStatus::default()));
    run_cycle(&config, RefreshMode::Fast, true, false, true, &status);
    let worker_status = Arc::clone(&status);
    thread::Builder::new()
        .name("economics-refresh".into())
        .spawn(move || refresh_loop(config, receiver, worker_status))
        .expect("automatic refresh worker must start");
    RefreshControl { sender, status }
}

fn refresh_loop(
    config: Config,
    receiver: Receiver<RefreshMode>,
    status: Arc<Mutex<RefreshStatus>>,
) {
    let crypto_interval = Duration::from_secs(config.crypto_refresh_seconds());
    let market_interval = Duration::from_secs(config.refresh_minutes.saturating_mul(60));
    let macro_interval = Duration::from_secs(config.macro_refresh_minutes().saturating_mul(60));
    let full_interval = Duration::from_secs(config.full_refresh_hours.saturating_mul(3_600));

    run_cycle(&config, RefreshMode::Full, true, true, false, &status);
    let mut last_market = Instant::now();
    let mut last_macro = Instant::now();
    let mut last_full = Instant::now();

    loop {
        let requested = match receiver.recv_timeout(crypto_interval) {
            Ok(mode) => Some(mode),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if matches!(requested, Some(RefreshMode::Full)) {
            with_status(&status, |state| state.queued = false);
            run_cycle(&config, RefreshMode::Full, true, true, false, &status);
            last_market = Instant::now();
            last_macro = Instant::now();
            last_full = Instant::now();
            continue;
        }

        if last_full.elapsed() >= full_interval {
            run_cycle(&config, RefreshMode::Full, true, true, false, &status);
            last_market = Instant::now();
            last_macro = Instant::now();
            last_full = Instant::now();
            continue;
        }

        let market_due = last_market.elapsed() >= market_interval;
        let macro_due = last_macro.elapsed() >= macro_interval;
        run_cycle(
            &config,
            RefreshMode::Fast,
            market_due,
            macro_due,
            false,
            &status,
        );
        if market_due {
            last_market = Instant::now();
        }
        if macro_due {
            last_macro = Instant::now();
        }
    }
}

fn run_cycle(
    config: &Config,
    mode: RefreshMode,
    market_due: bool,
    macro_due: bool,
    force_snapshot: bool,
    status: &Arc<Mutex<RefreshStatus>>,
) {
    with_status(status, |state| {
        state.running = true;
        state.mode = Some(mode);
        state.last_started_at = Some(now());
        state.phase = Some("starting".into());
        state.attempted = 0;
        state.stored = 0;
        state.unchanged = 0;
        state.errors.clear();
    });

    let result = collect_and_calculate(config, mode, market_due, macro_due, force_snapshot, status);
    with_status(status, |state| {
        state.running = false;
        state.mode = None;
        state.phase = None;
        state.last_completed_at = Some(now());
        match result {
            Ok(outcome) => {
                state.last_snapshot_as_of = Some(outcome.snapshot_as_of);
                state.attempted = outcome.report.attempted;
                state.stored = outcome.report.stored;
                state.unchanged = outcome.report.unchanged;
                state.errors = outcome.report.errors;
            }
            Err(error) => {
                state.errors = vec![error];
            }
        }
    });
}

struct RefreshOutcome {
    report: collectors::CollectionReport,
    snapshot_as_of: String,
}

fn collect_current_fred(config: &Config, db: &Db, start: &str) -> collectors::CollectionReport {
    let mut report = collectors::CollectionReport::default();
    for series in FRED_CURRENT_SERIES {
        match collectors::collect_fred(config, db, start, false, Some(series)) {
            Ok(collected) => report.merge(collected),
            Err(error) => report.errors.push(format!("{series}: {error}")),
        }
    }
    report
}

fn collect_and_calculate(
    config: &Config,
    mode: RefreshMode,
    market_due: bool,
    macro_due: bool,
    force_snapshot: bool,
    status: &Arc<Mutex<RefreshStatus>>,
) -> Result<RefreshOutcome, String> {
    let db = Db::open(&config.db_path).map_err(|error| format!("database: {error}"))?;
    let start =
        (Utc::now().date_naive() - ChronoDuration::days(INCREMENTAL_LOOKBACK_DAYS)).to_string();
    let mut report = collectors::CollectionReport::default();
    let mut observations_changed = false;

    update_progress(status, "binance-live", &report);
    merge_result(
        &mut report,
        "binance live",
        collectors::collect_binance_live(config, &db),
    );
    update_progress(status, "binance-live", &report);

    if market_due || matches!(mode, RefreshMode::Full) {
        if config.krx_api_key.is_some() {
            let before = report.stored;
            update_progress(status, "krx-latest", &report);
            merge_result(
                &mut report,
                "krx latest",
                live_market::collect_krx_fast(config, &db),
            );
            observations_changed |= report.stored > before;
            update_progress(status, "krx-latest", &report);
        }
        // The synchronous startup prime waits only for live BTC and newest KRX rows.
        // Slower history/macro feeds run immediately afterwards on the background worker.
        if !force_snapshot {
            let before = report.stored;
            update_progress(status, "binance-hourly", &report);
            merge_result(
                &mut report,
                "binance hourly",
                collectors::collect_binance(config, &db),
            );
            observations_changed |= report.stored > before;
            update_progress(status, "binance-hourly", &report);
            if config.fred_api_key.is_some() {
                let before = report.stored;
                update_progress(status, "fred-current", &report);
                report.merge(collect_current_fred(config, &db, &start));
                observations_changed |= report.stored > before;
                update_progress(status, "fred-current", &report);
            }
            let before = report.stored;
            update_progress(status, "treasury-auctions", &report);
            merge_result(
                &mut report,
                "treasury",
                collectors::collect_treasury(config, &db),
            );
            observations_changed |= report.stored > before;
            update_progress(status, "treasury-auctions", &report);
        }
    }

    if (macro_due || matches!(mode, RefreshMode::Full)) && config.ecos_api_key.is_some() {
        let before = report.stored;
        update_progress(status, "bok-ecos", &report);
        merge_result(
            &mut report,
            "ecos",
            collectors::collect_ecos(config, &db, None),
        );
        observations_changed |= report.stored > before;
        update_progress(status, "bok-ecos", &report);
    }

    if matches!(mode, RefreshMode::Full) {
        let before = report.stored;
        let official_start = if db
            .latest("bis", "GLOBAL_DOLLAR_CREDIT", None)
            .map_err(|error| format!("database: {error}"))?
            .is_some()
            && db
                .latest("ofr_fsi", "OFR_FSI", None)
                .map_err(|error| format!("database: {error}"))?
                .is_some()
            && db
                .latest("nyfed", "DEALER_FAILS", None)
                .map_err(|error| format!("database: {error}"))?
                .is_some()
        {
            (Utc::now().date_naive() - ChronoDuration::days(400)).to_string()
        } else {
            "2000-01-01".into()
        };
        update_progress(status, "built-in-official-feeds", &report);
        merge_result(
            &mut report,
            "built-in official feeds",
            collectors::collect_builtin_official(config, &db, &official_start),
        );
        observations_changed |= report.stored > before;
        update_progress(status, "built-in-official-feeds", &report);
        if config.krx_api_key.is_some() {
            let before = report.stored;
            update_progress(status, "krx-all-31-services", &report);
            merge_result(
                &mut report,
                "krx history",
                collectors::collect_krx(config, &db, None),
            );
            observations_changed |= report.stored > before;
            update_progress(status, "krx-all-31-services", &report);
            let before = report.stored;
            update_progress(status, "krx-latest-confirmation", &report);
            merge_result(
                &mut report,
                "krx latest",
                live_market::collect_krx_fast(config, &db),
            );
            observations_changed |= report.stored > before;
            update_progress(status, "krx-latest-confirmation", &report);
        }
        let before = report.stored;
        update_progress(status, "official-adapters", &report);
        merge_result(
            &mut report,
            "official adapters",
            collectors::collect_configured_adapters(config, &db),
        );
        observations_changed |= report.stored > before;
        update_progress(status, "official-adapters", &report);
        // ALFRED is historical/revision data, not a live feed. It is intentionally excluded
        // from automatic refresh so vintage endpoint failures cannot make current data look stale.
        // Use `collect-alfred` explicitly before historical backtests when required.
    }

    let snapshot_as_of = if force_snapshot
        || observations_changed
        || db
            .latest_snapshot_as_of()
            .map_err(|error| format!("database: {error}"))?
            .is_none()
    {
        update_progress(status, "rule-engine", &report);
        engine::run(&db, config)
            .map_err(|error| format!("rule engine: {error}"))?
            .as_of
    } else {
        db.latest_snapshot_as_of()
            .map_err(|error| format!("database: {error}"))?
            .ok_or_else(|| "no snapshot is available".to_string())?
    };
    Ok(RefreshOutcome {
        report,
        snapshot_as_of,
    })
}

fn update_progress(
    status: &Arc<Mutex<RefreshStatus>>,
    phase: &str,
    report: &collectors::CollectionReport,
) {
    with_status(status, |state| {
        state.phase = Some(phase.into());
        state.attempted = report.attempted;
        state.stored = report.stored;
        state.unchanged = report.unchanged;
        state.errors = report.errors.clone();
    });
}

fn merge_result(
    report: &mut collectors::CollectionReport,
    name: &str,
    result: Result<collectors::CollectionReport, Box<dyn std::error::Error>>,
) {
    match result {
        Ok(collected) => report.merge(collected),
        Err(error) => report.errors.push(format!("{name}: {error}")),
    }
}

fn with_status(status: &Arc<Mutex<RefreshStatus>>, update: impl FnOnce(&mut RefreshStatus)) {
    if let Ok(mut status) = status.lock() {
        update(&mut status);
    }
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_status_is_safe_to_expose_as_json() {
        let status = RefreshStatus {
            running: true,
            mode: Some(RefreshMode::Full),
            attempted: 10,
            stored: 2,
            ..RefreshStatus::default()
        };
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["running"], true);
        assert_eq!(json["mode"], "full");
        assert_eq!(json["stored"], 2);
    }

    #[test]
    fn discontinued_eqta_is_not_polled_by_live_refresh() {
        assert!(!FRED_CURRENT_SERIES.contains(&"EQTA"));
    }
}
