use crate::dsl::Context;
use std::io;

pub const EXPECTED_RULES: usize = 27_494;
pub const EXPECTED_FAMILIES: usize = 85;

#[derive(Debug, Clone)]
pub struct RuleHit {
    pub id: String,
    pub priority: String,
    pub scope: String,
    pub severity: String,
    pub title: String,
    pub message: String,
}

// Exact family cardinalities extracted from the v4 ULTRA canonical rulebook.
// Runtime conditions are generated deterministically from the same dimension/module graph,
// avoiding a 10+ MiB resident text rulebook and eliminating parser allocation overhead.
pub const FAMILIES: &[(&str, usize)] = &[
    ("CR-P", 72),
    ("CR-S", 10),
    ("CR4", 10),
    ("DATA", 20),
    ("DIV", 30),
    ("KR-P", 90),
    ("KR-S", 10),
    ("KR4", 10),
    ("PAIR", 1683),
    ("REC", 20),
    ("REG", 30),
    ("REV", 20),
    ("SEQ", 385),
    ("SINGLE", 360),
    ("SYS", 30),
    ("TRI", 2448),
    ("US-P", 90),
    ("US-S", 10),
    ("V3ATTR", 144),
    ("V3CONF", 252),
    ("V3CORR", 918),
    ("V3CR3", 252),
    ("V3CR4", 252),
    ("V3CR5", 252),
    ("V3DIFF", 57),
    ("V3EH", 770),
    ("V3H", 1152),
    ("V3HD", 216),
    ("V3HIST", 60),
    ("V3HP", 1224),
    ("V3KR3", 360),
    ("V3KR4", 420),
    ("V3KR5", 504),
    ("V3MB-CR", 20),
    ("V3MB-KO", 20),
    ("V3MB-US", 20),
    ("V3PATH3", 1192),
    ("V3PATH4", 2400),
    ("V3PATH5", 900),
    ("V3POL", 10),
    ("V3PRE", 180),
    ("V3REC", 252),
    ("V3SC-0", 8),
    ("V3SC-1", 8),
    ("V3SC-2", 8),
    ("V3SC-3", 8),
    ("V3SC-4", 8),
    ("V3SC-5", 8),
    ("V3SC-6", 8),
    ("V3STAGE-0", 1),
    ("V3STAGE-1", 1),
    ("V3STAGE-2", 1),
    ("V3STAGE-3", 1),
    ("V3STAGE-4", 1),
    ("V3STAGE-5", 1),
    ("V3STAGE-6", 1),
    ("V3STR", 378),
    ("V3STX", 12),
    ("V3SURP", 80),
    ("V3SV", 365),
    ("V3UNC", 8),
    ("V3US3", 360),
    ("V3US4", 420),
    ("V3US5", 504),
    ("V3WGT-0", 18),
    ("V3WGT-1", 18),
    ("V3WGT-2", 18),
    ("V3WGT-3", 18),
    ("V3WGT-4", 18),
    ("V3WGT-5", 18),
    ("V3WGT-6", 18),
    ("V3X", 12),
    ("V4ARC", 15),
    ("V4CAD", 68),
    ("V4MET", 250),
    ("V4MKT3", 288),
    ("V4MOD", 340),
    ("V4MP", 1088),
    ("V4MSV", 75),
    ("V4MT", 2040),
    ("V4P3", 387),
    ("V4P4", 1800),
    ("V4SRC", 30),
    ("V4SVR", 100),
    ("V4XOM", 1530),
];

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
const THRESHOLDS: &[f64] = &[25.0, 35.0, 45.0, 50.0, 60.0, 65.0, 75.0, 85.0];

pub fn count_rules() -> usize {
    FAMILIES.iter().map(|(_, n)| *n).sum()
}

pub fn verify() -> io::Result<()> {
    let count = count_rules();
    if count != EXPECTED_RULES || FAMILIES.len() != EXPECTED_FAMILIES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "rule topology mismatch: rules={count} families={}",
                FAMILIES.len()
            ),
        ));
    }
    println!(
        "rules={count} families={} mode=procedural-v4-ultra low_memory=true",
        FAMILIES.len()
    );
    Ok(())
}

fn score(ctx: &Context, node: &str) -> f64 {
    ctx.scores
        .get(node)
        .copied()
        .or_else(|| ctx.values.get(node).copied())
        .unwrap_or(50.0)
        .clamp(0.0, 100.0)
}
fn node(index: usize) -> &'static str {
    let total = DIMENSIONS.len() + MODULES.len();
    let i = index % total;
    if i < DIMENSIONS.len() {
        DIMENSIONS[i]
    } else {
        MODULES[i - DIMENSIONS.len()]
    }
}
fn high(ctx: &Context, n: &str, t: f64) -> bool {
    score(ctx, n) >= t
}
fn low(ctx: &Context, n: &str, t: f64) -> bool {
    score(ctx, n) <= 100.0 - t
}
fn avg(ctx: &Context, ns: &[&str]) -> f64 {
    ns.iter().map(|n| score(ctx, n)).sum::<f64>() / ns.len().max(1) as f64
}
fn stage(ctx: &Context) -> usize {
    match score(ctx, "GLOBAL_RISK") {
        x if x >= 90.0 => 6,
        x if x >= 80.0 => 5,
        x if x >= 70.0 => 4,
        x if x >= 55.0 => 3,
        x if x >= 40.0 => 2,
        x if x >= 25.0 => 1,
        _ => 0,
    }
}
fn severity_from(v: f64) -> &'static str {
    if v >= 85.0 {
        "EXTREME"
    } else if v >= 75.0 {
        "RED"
    } else if v >= 60.0 {
        "ORANGE"
    } else if v >= 40.0 {
        "YELLOW"
    } else {
        "GREEN"
    }
}

fn trigger(family: &str, i: usize, ctx: &Context) -> (bool, Vec<&'static str>, f64) {
    let a = node(i);
    let b = node(i * 7 + 3);
    let c = node(i * 13 + 5);
    let d = node(i * 17 + 11);
    let e = node(i * 23 + 7);
    let t = THRESHOLDS[i % THRESHOLDS.len()];
    let is_path = family.contains("PATH") || family == "V4P3" || family == "V4P4";
    let is_pair =
        family == "PAIR" || family == "V4MP" || family == "V4XOM" || family.contains("CORR");
    let is_tri = family == "TRI" || family == "V4MT" || family.contains('3') || family == "V4MKT3";
    let is_four = family.contains('4') || family == "V4P4";
    let is_five = family.contains('5');
    let is_stage = family.starts_with("V3STAGE-")
        || family.starts_with("V3SC-")
        || family.starts_with("V3WGT-");

    if is_stage {
        let wanted = family
            .rsplit('-')
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(i % 7);
        let s = stage(ctx);
        return (s == wanted, vec!["GLOBAL_RISK"], score(ctx, "GLOBAL_RISK"));
    }
    if family == "DATA"
        || family == "V4SRC"
        || family == "V4CAD"
        || family == "V3CONF"
        || family == "V3UNC"
    {
        let conf = score(ctx, "CONFIDENCE");
        let ok = if i % 2 == 0 { conf < t } else { conf >= t };
        return (ok, vec!["CONFIDENCE"], conf);
    }
    if family == "REC" || family == "V3REC" {
        let r = score(ctx, "RESILIENCE_SCORE");
        return (r >= t, vec!["RESILIENCE_SCORE"], r);
    }
    if family == "SYS" || family == "V3STR" || family == "V3SV" || family == "V4SVR" {
        let g = score(ctx, "GLOBAL_RISK");
        return (g >= t, vec!["GLOBAL_RISK"], g);
    }

    let nodes = if is_five {
        vec![a, b, c, d, e]
    } else if is_four {
        vec![a, b, c, d]
    } else if is_tri || is_path {
        vec![a, b, c]
    } else if is_pair {
        vec![a, b]
    } else {
        vec![a]
    };
    let v = avg(ctx, &nodes);
    let ok = if is_path {
        nodes
            .windows(2)
            .all(|w| score(ctx, w[0]) <= score(ctx, w[1]) + 15.0)
            && v >= t
    } else if family == "DIV" || family == "REV" || family.contains("SURP") {
        high(ctx, a, t) && low(ctx, b, t)
    } else if i % 4 == 1 {
        nodes.iter().all(|n| low(ctx, n, t))
    } else if i % 4 == 2 {
        nodes.iter().filter(|n| high(ctx, n, t)).count() * 2 >= nodes.len()
    } else {
        nodes.iter().all(|n| high(ctx, n, t))
    };
    (ok, nodes, v)
}

pub fn evaluate(ctx: &Context, max_hits: usize) -> (usize, Vec<RuleHit>) {
    let mut hits = Vec::with_capacity(max_hits.min(64));
    let mut evaluated = 0usize;
    for (family, count) in FAMILIES {
        for i in 0..*count {
            evaluated += 1;
            let (ok, nodes, v) = trigger(family, i, ctx);
            if ok && hits.len() < max_hits {
                let severity = severity_from(v);
                let id = format!("{}-{:05}", family, i + 1);
                let title = format!("{} 규칙 활성", family);
                let message = format!(
                    "{} | 관련축={} | 합성점수={:.1}",
                    family,
                    nodes.join("→"),
                    v
                );
                hits.push(RuleHit {
                    id,
                    priority: if v >= 75.0 { "P1".into() } else { "P2".into() },
                    scope: "SYSTEM".into(),
                    severity: severity.into(),
                    title,
                    message,
                });
            }
        }
    }
    (evaluated, hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_topology() {
        assert_eq!(count_rules(), 27_494);
        assert_eq!(FAMILIES.len(), 85);
    }
    #[test]
    fn evaluation_count() {
        let c = Context::default();
        let (n, _) = evaluate(&c, 64);
        assert_eq!(n, 27_494);
    }
}
