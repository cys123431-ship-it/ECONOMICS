use crate::{collectors, config::Config, db::Db, engine};
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
    let fast_interval = Duration::from_secs(config.refresh_minutes.saturating_mul(60));
    let full_interval = Duration::from_secs(config.full_refresh_hours.saturating_mul(3_600));
    let mut mode = RefreshMode::Full;
    let mut last_full = Instant::now();

    loop {
        run_cycle(&config, mode, &status);
        if matches!(mode, RefreshMode::Full) {
            last_full = Instant::now();
        }

        mode = match receiver.recv_timeout(fast_interval) {
            Ok(requested) => {
                with_status(&status, |state| state.queued = false);
                requested
            }
            Err(mpsc::RecvTimeoutError::Timeout) if last_full.elapsed() >= full_interval => {
                RefreshMode::Full
            }
            Err(mpsc::RecvTimeoutError::Timeout) => RefreshMode::Fast,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
    }
}

fn run_cycle(config: &Config, mode: RefreshMode, status: &Arc<Mutex<RefreshStatus>>) {
    with_status(status, |state| {
        state.running = true;
        state.mode = Some(mode);
        state.last_started_at = Some(now());
        state.errors.clear();
    });

    let result = collect_and_calculate(config, mode);
    with_status(status, |state| {
        state.running = false;
        state.mode = None;
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

fn collect_and_calculate(config: &Config, mode: RefreshMode) -> Result<RefreshOutcome, String> {
    let db = Db::open(&config.db_path).map_err(|error| format!("database: {error}"))?;
    let start =
        (Utc::now().date_naive() - ChronoDuration::days(INCREMENTAL_LOOKBACK_DAYS)).to_string();
    let mut report = collectors::CollectionReport::default();

    if config.fred_api_key.is_some() {
        merge_result(
            &mut report,
            "fred",
            collectors::collect_fred(config, &db, &start, false, None),
        );
        if matches!(mode, RefreshMode::Full) {
            merge_result(
                &mut report,
                "alfred",
                collectors::collect_fred(config, &db, &start, true, None),
            );
        }
    }
    merge_result(
        &mut report,
        "treasury",
        collectors::collect_treasury(config, &db),
    );
    merge_result(
        &mut report,
        "binance",
        collectors::collect_binance(config, &db),
    );

    if matches!(mode, RefreshMode::Full) {
        if config.ecos_api_key.is_some() {
            merge_result(
                &mut report,
                "ecos",
                collectors::collect_ecos(config, &db, None),
            );
        }
        if config.krx_api_key.is_some() {
            merge_result(
                &mut report,
                "krx",
                collectors::collect_krx(config, &db, None),
            );
        }
        merge_result(
            &mut report,
            "official adapters",
            collectors::collect_configured_adapters(config, &db),
        );
    }

    let snapshot = engine::run(&db, config).map_err(|error| format!("rule engine: {error}"))?;
    Ok(RefreshOutcome {
        report,
        snapshot_as_of: snapshot.as_of,
    })
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
}
