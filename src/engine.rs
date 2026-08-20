use crate::{
    config::Config,
    db::{Db, Point},
    dsl::{Context, Signal, SourceState},
    rulebook::{self, RuleHit},
    scoring,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Axis {
    Stress,
    Vulnerability,
    Resilience,
}

#[derive(Clone, Copy, Debug)]
enum Transform {
    Level,
    Absolute,
    PercentChange,
    DistanceOne,
}

#[derive(Clone, Copy, Debug)]
struct MetricDef {
    source: &'static str,
    series: &'static str,
    node: &'static str,
    axis: Axis,
    high_is_safe: bool,
    transform: Transform,
    weight: f64,
    max_age_days: i64,
    redundancy_group: &'static str,
}

const METRICS: &[MetricDef] = &[
    metric(
        "fred",
        "WEI",
        "GROWTH",
        Axis::Vulnerability,
        true,
        Transform::Level,
        1.0,
        14,
        "growth-nowcast",
    ),
    metric(
        "fred",
        "CFNAI",
        "GROWTH",
        Axis::Vulnerability,
        true,
        Transform::Level,
        1.0,
        60,
        "growth-nowcast",
    ),
    metric(
        "fred",
        "SAHMREALTIME",
        "LABOR",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        45,
        "labor",
    ),
    metric(
        "fred",
        "ICSA",
        "LABOR",
        Axis::Vulnerability,
        false,
        Transform::Level,
        0.8,
        14,
        "labor",
    ),
    metric(
        "fred",
        "CCSA",
        "LABOR",
        Axis::Vulnerability,
        false,
        Transform::Level,
        0.7,
        14,
        "labor",
    ),
    metric(
        "fred",
        "STLFSI4",
        "FINCOND",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        14,
        "financial-conditions",
    ),
    metric(
        "fred",
        "NFCI",
        "FINCOND",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        14,
        "financial-conditions",
    ),
    metric(
        "fred",
        "ANFCI",
        "FINCOND",
        Axis::Stress,
        false,
        Transform::Level,
        0.8,
        14,
        "financial-conditions",
    ),
    metric(
        "fred",
        "NFCILEVERAGE",
        "LEVERAGE",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        14,
        "leverage",
    ),
    metric(
        "fred",
        "BAMLH0A0HYM2",
        "CREDIT",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        7,
        "credit-spreads",
    ),
    metric(
        "fred",
        "BAMLC0A0CM",
        "CREDIT",
        Axis::Stress,
        false,
        Transform::Level,
        0.8,
        7,
        "credit-spreads",
    ),
    metric(
        "fred",
        "T10Y2Y",
        "RATES",
        Axis::Vulnerability,
        true,
        Transform::Level,
        1.0,
        7,
        "curve",
    ),
    metric(
        "fred",
        "T10Y3M",
        "RATES",
        Axis::Vulnerability,
        true,
        Transform::Level,
        0.8,
        7,
        "curve",
    ),
    metric(
        "fred",
        "VIXCLS",
        "VOLATILITY",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        7,
        "volatility",
    ),
    metric(
        "fred",
        "DEXKOUS",
        "USD",
        Axis::Stress,
        false,
        Transform::PercentChange,
        0.8,
        7,
        "usd",
    ),
    metric(
        "fred",
        "DTWEXBGS",
        "USD",
        Axis::Stress,
        false,
        Transform::PercentChange,
        1.0,
        7,
        "usd",
    ),
    metric(
        "fred",
        "MORTGAGE30US",
        "HOUSING",
        Axis::Vulnerability,
        false,
        Transform::Level,
        0.7,
        14,
        "household-rates",
    ),
    metric(
        "fred",
        "DRCCLACBS",
        "US_HOUSEHOLD_DEBT",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        120,
        "household-delinquency",
    ),
    metric(
        "fred",
        "DRCLACBS",
        "BUSINESS_DEBT",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        120,
        "business-delinquency",
    ),
    metric(
        "fred",
        "BUSLOANS",
        "BUSINESS_DEBT",
        Axis::Vulnerability,
        false,
        Transform::PercentChange,
        0.7,
        14,
        "business-credit",
    ),
    metric(
        "fred",
        "TOTLL",
        "BUSINESS_DEBT",
        Axis::Vulnerability,
        false,
        Transform::PercentChange,
        0.6,
        14,
        "business-credit",
    ),
    metric(
        "fred",
        "WALCL",
        "LIQUIDITY",
        Axis::Resilience,
        true,
        Transform::Level,
        0.7,
        14,
        "policy-liquidity",
    ),
    metric(
        "fred",
        "RRPONTSYD",
        "LIQUIDITY",
        Axis::Resilience,
        true,
        Transform::Level,
        0.5,
        7,
        "policy-liquidity",
    ),
    metric(
        "fred",
        "TOTRESNS",
        "BANKING",
        Axis::Resilience,
        true,
        Transform::Level,
        0.8,
        30,
        "bank-reserves",
    ),
    metric(
        "fred",
        "WRESBAL",
        "BANKING",
        Axis::Resilience,
        true,
        Transform::Level,
        0.8,
        14,
        "bank-reserves",
    ),
    metric(
        "fred",
        "EQTA",
        "BANKING",
        Axis::Resilience,
        true,
        Transform::Level,
        1.0,
        120,
        "bank-capital",
    ),
    metric(
        "fred",
        "KORLOLITOAASTSAM",
        "KOREA_MACRO",
        Axis::Vulnerability,
        true,
        Transform::Level,
        0.7,
        60,
        "korea-leading",
    ),
    metric(
        "fred",
        "CHNLOLITOAASTSAM",
        "CHINA_SPILLOVER",
        Axis::Vulnerability,
        true,
        Transform::Level,
        0.7,
        60,
        "china-leading",
    ),
    metric(
        "treasury",
        "AUCTION_BTC",
        "TREASURY_AUCTION",
        Axis::Stress,
        true,
        Transform::Level,
        1.0,
        21,
        "treasury-auction",
    ),
    metric(
        "treasury",
        "AUCTION_DEALER_SHARE",
        "TREASURY_AUCTION",
        Axis::Stress,
        false,
        Transform::Level,
        0.8,
        21,
        "treasury-auction",
    ),
    metric(
        "treasury",
        "AUCTION_INDIRECT_SHARE",
        "FOREIGN_TREASURY_DEMAND",
        Axis::Vulnerability,
        true,
        Transform::Level,
        0.8,
        21,
        "treasury-demand",
    ),
    metric(
        "binance",
        "BTC_FUNDING_ABS",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        2,
        "crypto-leverage",
    ),
    metric(
        "binance",
        "BTC_OI",
        "CRYPTO_DERIVATIVES",
        Axis::Vulnerability,
        false,
        Transform::PercentChange,
        1.0,
        2,
        "crypto-leverage",
    ),
    metric(
        "binance",
        "BTC_GLOBAL_LONG_SHORT",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::DistanceOne,
        0.6,
        2,
        "crypto-positioning",
    ),
    metric(
        "binance",
        "BTC_TOP_POSITION_RATIO",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::DistanceOne,
        0.6,
        2,
        "crypto-positioning",
    ),
    metric(
        "binance",
        "BTC_TOP_ACCOUNT_RATIO",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::DistanceOne,
        0.5,
        2,
        "crypto-positioning",
    ),
    metric(
        "binance",
        "BTC_TAKER_RATIO",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::DistanceOne,
        0.5,
        2,
        "crypto-flow",
    ),
    metric(
        "binance",
        "BTC_BASIS_ABS",
        "CRYPTO_DERIVATIVES",
        Axis::Stress,
        false,
        Transform::Level,
        0.7,
        2,
        "crypto-basis",
    ),
    metric(
        "ecos",
        "KR_USD_KRW",
        "KOREA_FIN_STAB",
        Axis::Stress,
        false,
        Transform::PercentChange,
        1.0,
        7,
        "korea-fx",
    ),
    metric(
        "ecos",
        "KR_BASE_RATE",
        "KOREA_FIN_STAB",
        Axis::Vulnerability,
        false,
        Transform::Level,
        0.7,
        45,
        "korea-rates",
    ),
    metric(
        "krx",
        "KRX_FOREIGN_NET_BUY",
        "KOREA_MARKET_INTERNALS",
        Axis::Stress,
        true,
        Transform::Level,
        1.0,
        3,
        "krx-flow",
    ),
    metric(
        "krx",
        "KRX_SHORT_BALANCE",
        "KOREA_MARKET_INTERNALS",
        Axis::Stress,
        false,
        Transform::Level,
        0.8,
        3,
        "krx-short",
    ),
    metric(
        "krx",
        "KRX_FUTURES_BASIS",
        "KOREA_MARKET_INTERNALS",
        Axis::Stress,
        false,
        Transform::Absolute,
        0.7,
        3,
        "krx-derivatives",
    ),
    metric(
        "krx",
        "KRX_PUT_CALL_RATIO",
        "KOREA_MARKET_INTERNALS",
        Axis::Stress,
        false,
        Transform::Level,
        0.7,
        3,
        "krx-derivatives",
    ),
    metric(
        "krx",
        "KRX_BREADTH",
        "KOREA_MARKET_INTERNALS",
        Axis::Stress,
        true,
        Transform::Level,
        0.8,
        3,
        "krx-breadth",
    ),
    metric(
        "ofr_fsi",
        "OFR_FSI",
        "FINCOND",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        4,
        "financial-conditions",
    ),
    metric(
        "ofr_repo",
        "SOFR_P99_SPREAD",
        "REPO_MICRO",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        3,
        "repo",
    ),
    metric(
        "nyfed",
        "DEALER_FAILS",
        "DEALER_INTERMEDIATION",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        10,
        "dealer",
    ),
    metric(
        "scoos",
        "MARGIN_TIGHTENING",
        "MARGIN_COLLATERAL",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        120,
        "margin",
    ),
    metric(
        "ofr_hfm",
        "HF_LEVERAGE",
        "HEDGE_FUND_LEVERAGE",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        120,
        "hedge-fund",
    ),
    metric(
        "ofr_hfm",
        "HF_OVERNIGHT_FUNDING",
        "HEDGE_FUND_FUNDING",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        120,
        "hedge-fund",
    ),
    metric(
        "ofr_hfm",
        "HF_COUNTERPARTY_CONCENTRATION",
        "HF_COUNTERPARTY",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        120,
        "hedge-fund",
    ),
    metric(
        "ofr_mmf",
        "MMF_REPO_CONCENTRATION",
        "MMF_FUNDING",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        45,
        "mmf",
    ),
    metric(
        "treasury",
        "BUYBACK_OFFER_ACCEPT_RATIO",
        "TREASURY_BUYBACK_FLOW",
        Axis::Stress,
        false,
        Transform::Level,
        1.0,
        21,
        "treasury-buyback",
    ),
    metric(
        "treasury_tic",
        "FOREIGN_TREASURY_FLOW",
        "FOREIGN_TREASURY_DEMAND",
        Axis::Vulnerability,
        true,
        Transform::Level,
        1.0,
        50,
        "treasury-demand",
    ),
    metric(
        "bis",
        "GLOBAL_DOLLAR_CREDIT",
        "GLOBAL_DOLLAR_CREDIT",
        Axis::Vulnerability,
        false,
        Transform::PercentChange,
        1.0,
        130,
        "global-dollar",
    ),
    metric(
        "fsc",
        "KOREA_HOUSEHOLD_PF_RISK",
        "KOREA_FIN_STAB",
        Axis::Vulnerability,
        false,
        Transform::Level,
        1.0,
        60,
        "korea-financial-stability",
    ),
];

#[allow(clippy::too_many_arguments)]
const fn metric(
    source: &'static str,
    series: &'static str,
    node: &'static str,
    axis: Axis,
    high_is_safe: bool,
    transform: Transform,
    weight: f64,
    max_age_days: i64,
    redundancy_group: &'static str,
) -> MetricDef {
    MetricDef {
        source,
        series,
        node,
        axis,
        high_is_safe,
        transform,
        weight,
        max_age_days,
        redundancy_group,
    }
}

pub const DIMENSIONS: &[&str] = &[
    "GROWTH",
    "LABOR",
    "INFLATION",
    "RATES",
    "TREASURY",
    "CREDIT",
    "BANKING",
    "LIQUIDITY",
    "FINCOND",
    "LEVERAGE",
    "VOLATILITY",
    "USD",
    "HOUSING",
    "CONSUMER",
    "CORP_HEALTH",
    "FUNDING",
    "KOREA_MACRO",
    "CHINA_SPILLOVER",
];

pub const MODULES: &[&str] = &[
    "VALUATION",
    "BUSINESS_DEBT",
    "US_HOUSEHOLD_DEBT",
    "HEDGE_FUND_LEVERAGE",
    "HEDGE_FUND_FUNDING",
    "HF_COUNTERPARTY",
    "DEALER_INTERMEDIATION",
    "REPO_MICRO",
    "MMF_FUNDING",
    "MARGIN_COLLATERAL",
    "TREASURY_AUCTION",
    "TREASURY_BUYBACK_FLOW",
    "FOREIGN_TREASURY_DEMAND",
    "GLOBAL_DOLLAR_CREDIT",
    "KOREA_FIN_STAB",
    "KOREA_MARKET_INTERNALS",
    "CRYPTO_DERIVATIVES",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceDiagnostic {
    pub latest_observed_at: Option<String>,
    pub latest_released_at: Option<String>,
    pub latest_ingested_at: Option<String>,
    pub age_days: Option<i64>,
    pub fresh: bool,
    pub revisions: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub ts: String,
    pub as_of: String,
    pub global_risk: Option<f64>,
    pub stress: Option<f64>,
    pub vulnerability: Option<f64>,
    pub resilience: Option<f64>,
    pub confidence: f64,
    pub data_quality: f64,
    pub diffusion: usize,
    pub stage: Option<u8>,
    pub markets: HashMap<String, Option<f64>>,
    pub nodes: HashMap<String, f64>,
    pub sources: HashMap<String, SourceDiagnostic>,
    pub causes: Vec<String>,
    pub rule_hits: Vec<RuleHit>,
    pub rules_evaluated: usize,
    pub rules_triggered: usize,
    pub rules_indeterminate: usize,
}

#[derive(Default)]
struct NodeAccumulator {
    groups: HashMap<&'static str, Vec<(f64, f64)>>,
    histories: Vec<Vec<f64>>,
    sources: HashSet<&'static str>,
    axes: HashSet<Axis>,
}

pub fn run(db: &Db, config: &Config) -> Result<Snapshot, Box<dyn std::error::Error>> {
    run_at(db, config, &Utc::now().to_rfc3339(), true)
}

pub fn run_at(
    db: &Db,
    config: &Config,
    as_of: &str,
    save: bool,
) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let as_of_time = parse_time(as_of).ok_or("as_of must be RFC3339 or YYYY-MM-DD")?;
    let mut context = Context::default();
    let mut accumulators: HashMap<&'static str, NodeAccumulator> = HashMap::new();
    let total_weight = METRICS.iter().map(|metric| metric.weight).sum::<f64>();
    let mut available_weight = 0.0;
    let mut fresh_weight = 0.0;

    for definition in METRICS {
        let prefer_alfred =
            definition.source == "fred" && as_of_time.date_naive() < Utc::now().date_naive();
        let mut query_source = if prefer_alfred {
            "alfred"
        } else {
            definition.source
        };
        let mut latest = db.latest(query_source, definition.series, Some(as_of))?;
        if latest.is_none() && query_source != definition.source {
            query_source = definition.source;
            latest = db.latest(query_source, definition.series, Some(as_of))?;
        }
        let Some(latest) = latest else {
            continue;
        };
        let points = db.recent(query_source, definition.series, 512, Some(as_of))?;
        if points.is_empty() {
            continue;
        }
        let age = age_days(&latest.observed_at, as_of_time);
        let is_fresh = age.is_some_and(|age| age <= definition.max_age_days);
        if !is_fresh {
            continue;
        }
        let transformed = transform_points(&points, definition.transform);
        if transformed.len() < config.min_samples + 1 {
            continue;
        }
        let current = *transformed.last().unwrap_or(&f64::NAN);
        let history = &transformed[..transformed.len() - 1];
        let Some(risk) = scoring::risk_from_history(
            history,
            current,
            definition.high_is_safe,
            config.min_samples,
        ) else {
            continue;
        };
        let risk_history =
            rolling_risk_history(&transformed, definition.high_is_safe, config.min_samples);
        available_weight += definition.weight;
        fresh_weight += definition.weight;
        context.insert_signal(
            definition.series,
            Signal {
                current: Some(current),
                history: transformed.clone(),
            },
        );
        context.insert_number(format!("METRIC_RISK:{}", definition.series), risk);
        let accumulator = accumulators.entry(definition.node).or_default();
        accumulator
            .groups
            .entry(definition.redundancy_group)
            .or_default()
            .push((risk, definition.weight));
        accumulator.histories.push(risk_history);
        accumulator.sources.insert(query_source);
        accumulator.axes.insert(definition.axis);
    }
    if let Some(breadth) = context.value("KRX_BREADTH") {
        context.insert_number("BREADTH", breadth);
        context.insert_bool("MARKET_BREADTH:KOREA_EQUITY", true);
    }

    let mut nodes = HashMap::new();
    let mut node_axes: HashMap<String, HashSet<Axis>> = HashMap::new();
    for (node, accumulator) in accumulators {
        let group_scores = accumulator
            .groups
            .values()
            .filter_map(|values| scoring::weighted_mean(values))
            .collect::<Vec<_>>();
        let Some(score) = scoring::mean(&group_scores) else {
            continue;
        };
        let history = aggregate_histories(&accumulator.histories);
        context.insert_signal(
            node,
            Signal {
                current: Some(score),
                history,
            },
        );
        if accumulator.sources.len() >= 2 {
            context.insert_number(
                format!("CROSS_SOURCE_CONFIRM:{node}"),
                (60.0 + accumulator.sources.len() as f64 * 10.0).min(100.0),
            );
        }
        nodes.insert(node.to_string(), score);
        node_axes.insert(node.to_string(), accumulator.axes);
    }
    enrich_node_context(&nodes, &mut context);

    let mut stress_nodes = Vec::new();
    let mut vulnerability_nodes = Vec::new();
    let mut resilience_nodes = Vec::new();
    for (node, score) in &nodes {
        if let Some(axes) = node_axes.get(node) {
            if axes.contains(&Axis::Stress) {
                stress_nodes.push(*score);
            }
            if axes.contains(&Axis::Vulnerability) {
                vulnerability_nodes.push(*score);
            }
            if axes.contains(&Axis::Resilience) {
                resilience_nodes.push(100.0 - *score);
            }
        }
    }
    let stress = scoring::mean(&stress_nodes).map(scoring::clamp);
    let vulnerability = scoring::mean(&vulnerability_nodes).map(scoring::clamp);
    let resilience = scoring::mean(&resilience_nodes).map(scoring::clamp);

    let expected_nodes = DIMENSIONS.len() + MODULES.len();
    let metric_coverage = available_weight / total_weight;
    let node_coverage = nodes.len() as f64 / expected_nodes as f64;
    let freshness = if available_weight > 0.0 {
        fresh_weight / available_weight
    } else {
        0.0
    };
    let confidence =
        scoring::clamp(100.0 * (0.55 * metric_coverage + 0.30 * node_coverage + 0.15 * freshness));
    let data_quality = scoring::clamp(100.0 * (0.65 * freshness + 0.35 * metric_coverage));
    let base_risk = match (stress, vulnerability) {
        (Some(stress), Some(vulnerability)) => Some(0.55 * stress + 0.45 * vulnerability),
        (Some(stress), None) => Some(stress),
        (None, Some(vulnerability)) => Some(vulnerability),
        (None, None) => None,
    };
    let global_risk = if confidence < 35.0 {
        None
    } else {
        base_risk.map(|risk| {
            scoring::clamp(risk + resilience.map(|value| 0.20 * (50.0 - value)).unwrap_or(0.0))
        })
    };

    for (name, value) in [
        ("STRESS_SCORE", stress),
        ("VULNERABILITY_SCORE", vulnerability),
        ("RESILIENCE_SCORE", resilience),
        ("GLOBAL_RISK", global_risk),
    ] {
        if let Some(value) = value {
            context.insert_number(name, value);
        }
    }
    context.insert_number("CONFIDENCE", confidence);
    context.insert_number("DATA_QUALITY", data_quality);
    context.insert_number("KNOWN_SIGNAL_COUNT", context.known_signal_count() as f64);
    context.insert_bool("ENGINE_HAS_GLOBAL_RISK", global_risk.is_some());
    context.insert_text(
        "ENGINE_STATUS",
        if global_risk.is_some() {
            "READY"
        } else {
            "INSUFFICIENT_DATA"
        },
    );

    let sources = build_source_diagnostics(db, as_of, as_of_time, &mut context)?;
    let previous = db
        .latest_snapshot_before(as_of)?
        .and_then(|payload| serde_json::from_str::<Snapshot>(&payload).ok());
    let stage = global_risk.map(|risk| stage_with_hysteresis(risk, previous.as_ref()));
    if let Some(stage) = stage {
        context.insert_number("CRISIS_STAGE", stage as f64);
        for dimension in DIMENSIONS.iter().chain(MODULES) {
            context.insert_text(
                format!("DYNAMIC_WEIGHT:STAGE={stage},DIMENSION={dimension}"),
                "ACTIVE",
            );
        }
    }

    let market = |node_names: &[&str]| {
        scoring::mean(
            &node_names
                .iter()
                .filter_map(|name| nodes.get(*name).copied())
                .collect::<Vec<_>>(),
        )
        .map(scoring::clamp)
    };
    let mut markets = HashMap::new();
    markets.insert(
        "US_EQUITY".into(),
        market(&["VALUATION", "CREDIT", "VOLATILITY", "FINCOND", "LEVERAGE"]),
    );
    markets.insert(
        "CRYPTO".into(),
        market(&["CRYPTO_DERIVATIVES", "LIQUIDITY", "USD"]),
    );
    markets.insert(
        "KOREA_EQUITY".into(),
        market(&[
            "KOREA_FIN_STAB",
            "KOREA_MARKET_INTERNALS",
            "KOREA_MACRO",
            "USD",
        ]),
    );
    for (name, value) in &markets {
        if let Some(value) = value {
            context.insert_number(format!("MKT_STRESS:{name}"), *value);
        }
    }

    let evaluation = rulebook::evaluate(&config.rulebook_path, &context, 64)?;
    let causes = evaluation
        .hits
        .iter()
        .map(|hit| {
            format!(
                "{} [{} {:?}]: {} — {}",
                hit.id, hit.severity, hit.channel, hit.title, hit.message
            )
        })
        .collect();
    let diffusion = nodes.values().filter(|score| **score >= 75.0).count();
    let snapshot = Snapshot {
        ts: Utc::now().to_rfc3339(),
        as_of: as_of.into(),
        global_risk,
        stress,
        vulnerability,
        resilience,
        confidence,
        data_quality,
        diffusion,
        stage,
        markets,
        nodes,
        sources,
        causes,
        rule_hits: evaluation.hits,
        rules_evaluated: evaluation.rules_evaluated,
        rules_triggered: evaluation.rules_triggered,
        rules_indeterminate: evaluation.rules_indeterminate,
    };
    if save {
        db.save_snapshot(&snapshot)?;
    }
    Ok(snapshot)
}

fn enrich_node_context(nodes: &HashMap<String, f64>, context: &mut Context) {
    let mut contributors = nodes.iter().collect::<Vec<_>>();
    contributors.sort_by(|left, right| right.1.total_cmp(left.1).then_with(|| left.0.cmp(right.0)));
    let top_three = contributors
        .iter()
        .take(3)
        .map(|(name, _)| name.as_str())
        .collect::<HashSet<_>>();
    for (name, score) in nodes {
        context.insert_number(format!("CONTRIB:{name}"), *score);
        context.insert_bool(
            format!("TOP3_CONTRIBUTOR:{name}"),
            top_three.contains(name.as_str()),
        );
    }

    const CLUSTERS: &[(&str, &[&str])] = &[
        (
            "REAL",
            &[
                "GROWTH",
                "LABOR",
                "INFLATION",
                "HOUSING",
                "CONSUMER",
                "CORP_HEALTH",
            ],
        ),
        (
            "FINANCIAL",
            &[
                "RATES",
                "TREASURY",
                "CREDIT",
                "BANKING",
                "LIQUIDITY",
                "FINCOND",
                "VOLATILITY",
                "FUNDING",
            ],
        ),
        (
            "FRAGILITY",
            &[
                "LEVERAGE",
                "BUSINESS_DEBT",
                "US_HOUSEHOLD_DEBT",
                "HEDGE_FUND_LEVERAGE",
                "HEDGE_FUND_FUNDING",
                "HF_COUNTERPARTY",
                "DEALER_INTERMEDIATION",
                "REPO_MICRO",
                "MMF_FUNDING",
                "MARGIN_COLLATERAL",
            ],
        ),
        ("POLICY", &["RATES", "LIQUIDITY", "INFLATION"]),
        (
            "EXTERNAL",
            &[
                "USD",
                "KOREA_MACRO",
                "CHINA_SPILLOVER",
                "FOREIGN_TREASURY_DEMAND",
                "GLOBAL_DOLLAR_CREDIT",
            ],
        ),
    ];
    for (cluster, members) in CLUSTERS {
        let values = members
            .iter()
            .filter_map(|name| nodes.get(*name).copied())
            .collect::<Vec<_>>();
        if values.is_empty() {
            continue;
        }
        let high = values.iter().filter(|value| **value >= 65.0).count();
        context.insert_number(
            format!("CLUSTER_BREADTH:{cluster}"),
            100.0 * high as f64 / values.len() as f64,
        );
        context.insert_number(
            format!("COUNT_HIGH:CLUSTER={cluster},THRESHOLD=65"),
            high as f64,
        );
    }
}

fn transform_points(points: &[Point], transform: Transform) -> Vec<f64> {
    let values = points.iter().map(|point| point.value).collect::<Vec<_>>();
    match transform {
        Transform::Level => values,
        Transform::Absolute => values.into_iter().map(f64::abs).collect(),
        Transform::DistanceOne => values
            .into_iter()
            .map(|value| (value - 1.0).abs())
            .collect(),
        Transform::PercentChange => values
            .windows(2)
            .filter(|window| window[0].abs() > f64::EPSILON)
            .map(|window| 100.0 * (window[1] / window[0] - 1.0))
            .collect(),
    }
}

fn rolling_risk_history(values: &[f64], invert: bool, min_samples: usize) -> Vec<f64> {
    let mut scores = Vec::new();
    for index in min_samples..values.len() {
        let start = index.saturating_sub(256);
        if let Some(score) = scoring::risk_from_history(
            &values[start..index],
            values[index],
            invert,
            min_samples.min(index - start),
        ) {
            scores.push(score);
        }
    }
    scores
}

fn aggregate_histories(histories: &[Vec<f64>]) -> Vec<f64> {
    let Some(length) = histories.iter().map(Vec::len).min() else {
        return Vec::new();
    };
    (0..length)
        .filter_map(|offset| {
            scoring::mean(
                &histories
                    .iter()
                    .map(|history| history[history.len() - length + offset])
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn build_source_diagnostics(
    db: &Db,
    as_of: &str,
    as_of_time: DateTime<Utc>,
    context: &mut Context,
) -> Result<HashMap<String, SourceDiagnostic>, Box<dyn std::error::Error>> {
    let specs = [
        ("fred", "FRED_DAILY", 4),
        ("alfred", "FRED_MONTHLY", 45),
        ("ofr_repo", "OFR_REPO", 3),
        ("ofr_fsi", "OFR_FSI", 4),
        ("nyfed", "NYFED_MARKETS", 3),
        ("scoos", "FED_SCOOS", 120),
        ("ofr_hfm", "OFR_HFM", 120),
        ("ofr_mmf", "OFR_MMF", 45),
        ("cftc", "CFTC_COT", 10),
        ("treasury", "TREASURY_AUCTION", 21),
        ("treasury_tic", "TREASURY_TIC", 50),
        ("bis", "BIS_GLI", 130),
        ("ecos", "BOK_ECOS", 60),
        ("krx", "KRX", 3),
        ("binance", "BINANCE", 2),
    ];
    let mut diagnostics = HashMap::new();
    for (source, canonical, max_age) in specs {
        let freshness = db.source_freshness(source, Some(as_of))?;
        let age = freshness
            .latest_observed_at
            .as_deref()
            .and_then(|value| age_days(value, as_of_time));
        let fresh = age.is_some_and(|age| age <= max_age);
        context.insert_source(
            canonical,
            SourceState {
                age_days: age.map(|age| age as f64),
                missing: freshness.latest_observed_at.is_none(),
                revised: freshness.revisions > 0,
                fresh,
            },
        );
        diagnostics.insert(
            canonical.into(),
            SourceDiagnostic {
                latest_observed_at: freshness.latest_observed_at,
                latest_released_at: freshness.latest_released_at,
                latest_ingested_at: freshness.latest_ingested_at,
                age_days: age,
                fresh,
                revisions: freshness.revisions,
            },
        );
    }
    Ok(diagnostics)
}

fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(&value[..value.len().min(10)], "%Y-%m-%d")
                .ok()?
                .and_hms_opt(23, 59, 59)
                .map(|value| value.and_utc())
        })
}

fn age_days(observed_at: &str, as_of: DateTime<Utc>) -> Option<i64> {
    parse_time(observed_at).map(|observed| (as_of - observed).num_days().max(0))
}

fn stage_with_hysteresis(risk: f64, previous: Option<&Snapshot>) -> u8 {
    let thresholds = [25.0, 40.0, 55.0, 70.0, 80.0, 90.0];
    let mut stage = thresholds
        .iter()
        .filter(|threshold| risk >= **threshold)
        .count() as u8;
    if let Some(previous_stage) = previous.and_then(|snapshot| snapshot.stage) {
        if stage < previous_stage {
            let release_threshold = if previous_stage == 0 {
                0.0
            } else {
                thresholds[previous_stage as usize - 1] - 5.0
            };
            if risk >= release_threshold {
                stage = previous_stage;
            }
        }
    }
    stage
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stage_requires_hysteresis_to_fall() {
        let previous = Snapshot {
            ts: String::new(),
            as_of: String::new(),
            global_risk: Some(72.0),
            stress: None,
            vulnerability: None,
            resilience: None,
            confidence: 100.0,
            data_quality: 100.0,
            diffusion: 0,
            stage: Some(4),
            markets: HashMap::new(),
            nodes: HashMap::new(),
            sources: HashMap::new(),
            causes: Vec::new(),
            rule_hits: Vec::new(),
            rules_evaluated: 0,
            rules_triggered: 0,
            rules_indeterminate: 0,
        };
        assert_eq!(stage_with_hysteresis(67.0, Some(&previous)), 4);
        assert_eq!(stage_with_hysteresis(64.0, Some(&previous)), 3);
    }

    #[test]
    fn percent_change_transform_does_not_mix_levels() {
        let points = [1.0, 2.0, 1.0]
            .iter()
            .enumerate()
            .map(|(index, value)| Point {
                observed_at: format!("2026-01-{:02}", index + 1),
                value: *value,
                released_at: None,
                source_asof: None,
                ingested_at: String::new(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            transform_points(&points, Transform::PercentChange),
            vec![100.0, -50.0]
        );
    }

    #[test]
    fn empty_database_never_fabricates_market_risk() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("empty.db")).unwrap();
        let config = Config {
            fred_api_key: None,
            ecos_api_key: None,
            krx_api_key: None,
            krx_api_url: None,
            official_adapters_file: None,
            db_path: temporary.path().join("empty.db"),
            rulebook_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("rulebook")
                .join(crate::config::CANONICAL_RULEBOOK_NAME),
            host: "127.0.0.1:0".into(),
            min_samples: 20,
            http_timeout_secs: 1,
        };
        let snapshot = run_at(&db, &config, "2026-08-20T23:59:59Z", false).unwrap();
        assert_eq!(snapshot.global_risk, None);
        assert_eq!(snapshot.stress, None);
        assert_eq!(snapshot.vulnerability, None);
        assert_eq!(snapshot.resilience, None);
        assert!(snapshot
            .rule_hits
            .iter()
            .all(|hit| hit.channel == rulebook::Channel::DataQuality));
    }
}
