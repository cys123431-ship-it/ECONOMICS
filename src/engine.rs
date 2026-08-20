use crate::{db::Db, dsl::Context, rulebook, scoring};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub ts: String,
    pub global_risk: f64,
    pub stress: f64,
    pub vulnerability: f64,
    pub resilience: f64,
    pub confidence: f64,
    pub diffusion: usize,
    pub stage: u8,
    pub markets: HashMap<String, f64>,
    pub causes: Vec<String>,
    pub rules_evaluated: usize,
    pub rule_hits: usize,
}

const METRICS: &[(&str, &str, &str, bool, &str)] = &[
    ("fred", "STLFSI4", "FINCOND", false, "stress"),
    ("fred", "NFCI", "FINCOND", false, "stress"),
    ("fred", "BAMLH0A0HYM2", "CREDIT", false, "stress"),
    ("fred", "VIXCLS", "VOLATILITY", false, "stress"),
    ("fred", "T10Y2Y", "RATES", true, "vulnerability"),
    ("fred", "NFCILEVERAGE", "LEVERAGE", false, "vulnerability"),
    ("fred", "ICSA", "LABOR", false, "vulnerability"),
    ("fred", "WEI", "GROWTH", true, "vulnerability"),
    (
        "binance",
        "BTC_FUNDING",
        "CRYPTO_DERIVATIVES",
        false,
        "stress",
    ),
    (
        "binance",
        "BTC_OI",
        "CRYPTO_DERIVATIVES",
        false,
        "vulnerability",
    ),
    (
        "treasury",
        "AUCTION_BTC",
        "TREASURY_AUCTION",
        true,
        "stress",
    ),
    ("ecos", "KR_USD_KRW", "KOREA_FIN_STAB", false, "stress"),
];

pub fn run(db: &Db) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let mut ctx = Context::default();
    let mut stress = Vec::new();
    let mut vulnerability = Vec::new();
    let mut confidence_parts = 0usize;
    let mut diffusion = 0usize;
    let mut causes = Vec::new();

    for (source, series, module, invert, bucket) in METRICS {
        if let Some(current) = db.latest(source, series)? {
            let history = db.recent(source, series, 256)?;
            let metric_score = scoring::risk_from_z_like(&history, current, *invert);
            ctx.values.insert((*series).into(), current);
            ctx.scores
                .entry((*module).into())
                .and_modify(|v| *v = (*v + metric_score) / 2.0)
                .or_insert(metric_score);
            confidence_parts += 1;
            if metric_score >= 75.0 {
                diffusion += 1;
                causes.push(format!("{module}:{metric_score:.0}"));
            }
            if *bucket == "stress" {
                stress.push(metric_score);
            } else {
                vulnerability.push(metric_score);
            }
        }
    }

    let stress_score = scoring::mean(&stress);
    let vulnerability_score = scoring::mean(&vulnerability);
    let resilience = scoring::clamp(100.0 - (0.35 * stress_score + 0.25 * vulnerability_score));
    let global_risk = scoring::clamp(
        0.55 * stress_score + 0.45 * vulnerability_score + 0.15 * (100.0 - resilience),
    );
    let confidence = (confidence_parts as f64 / METRICS.len() as f64 * 100.0).clamp(0.0, 100.0);

    ctx.scores.insert("STRESS_SCORE".into(), stress_score);
    ctx.scores
        .insert("VULNERABILITY_SCORE".into(), vulnerability_score);
    ctx.scores.insert("RESILIENCE_SCORE".into(), resilience);
    ctx.scores.insert("GLOBAL_RISK".into(), global_risk);
    ctx.scores.insert("CONFIDENCE".into(), confidence);

    let (rules_evaluated, hits) = rulebook::evaluate(&ctx, 64);
    for hit in &hits {
        causes.push(format!(
            "{} [{}]: {} — {}",
            hit.id, hit.severity, hit.title, hit.message
        ));
    }

    let stage = if global_risk >= 90.0 {
        6
    } else if global_risk >= 80.0 {
        5
    } else if global_risk >= 70.0 {
        4
    } else if global_risk >= 55.0 {
        3
    } else if global_risk >= 40.0 {
        2
    } else if global_risk >= 25.0 {
        1
    } else {
        0
    };

    let mut markets = HashMap::new();
    markets.insert(
        "US_EQUITY".into(),
        scoring::clamp(0.45 * stress_score + 0.55 * vulnerability_score),
    );
    markets.insert(
        "CRYPTO".into(),
        ctx.scores
            .get("CRYPTO_DERIVATIVES")
            .copied()
            .unwrap_or(global_risk),
    );
    markets.insert(
        "KOREA_EQUITY".into(),
        ctx.scores
            .get("KOREA_FIN_STAB")
            .copied()
            .unwrap_or(global_risk),
    );

    let snapshot = Snapshot {
        ts: chrono::Utc::now().to_rfc3339(),
        global_risk,
        stress: stress_score,
        vulnerability: vulnerability_score,
        resilience,
        confidence,
        diffusion,
        stage,
        markets,
        causes,
        rules_evaluated,
        rule_hits: hits.len(),
    };
    db.save_snapshot(&snapshot)?;
    Ok(snapshot)
}
