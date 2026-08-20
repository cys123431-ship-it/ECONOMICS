use crate::dsl::{self, Context, Truth};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::Path,
};

pub const EXPECTED_RULES: usize = 27_494;
pub const EXPECTED_FAMILIES: usize = 85;
pub const CANONICAL_SHA256: &str =
    "2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56";

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub priority: u16,
    pub scope: String,
    pub severity: String,
    pub condition: String,
    pub tags: Vec<String>,
    pub suppress: String,
    pub source: String,
    pub title: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Channel {
    Primary,
    Confirmation,
    CounterSignal,
    DataQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleHit {
    pub id: String,
    pub priority: u16,
    pub scope: String,
    pub severity: String,
    pub channel: Channel,
    pub title: String,
    pub message: String,
    pub condition: String,
    pub tags: Vec<String>,
    pub source: String,
    #[serde(skip)]
    suppress: String,
    #[serde(skip)]
    specificity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verification {
    pub sha256: String,
    pub rules: usize,
    pub families: usize,
    pub duplicate_ids: usize,
    pub invalid_conditions: usize,
    pub sources: HashMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evaluation {
    pub rules_evaluated: usize,
    pub rules_triggered: usize,
    pub rules_indeterminate: usize,
    pub hits: Vec<RuleHit>,
}

#[derive(Clone, Debug)]
struct PendingRule {
    id: String,
    priority: u16,
    scope: String,
    severity: String,
    condition: String,
    tags: Vec<String>,
    suppress: String,
    source: String,
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn for_each_rule(
    path: &Path,
    mut callback: impl FnMut(Rule) -> io::Result<()>,
) -> io::Result<usize> {
    let file = BufReader::new(File::open(path)?);
    let mut pending: Option<PendingRule> = None;
    let mut count = 0usize;
    for (line_index, line) in file.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line?;
        if line.starts_with("RULE\t") {
            if pending.is_some() {
                return Err(invalid(format!(
                    "line {line_number}: RULE encountered before prior MSG"
                )));
            }
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 9 {
                return Err(invalid(format!(
                    "line {line_number}: RULE has {} fields, expected 9",
                    fields.len()
                )));
            }
            let priority = fields[2].parse::<u16>().map_err(|_| {
                invalid(format!(
                    "line {line_number}: invalid priority {}",
                    fields[2]
                ))
            })?;
            pending = Some(PendingRule {
                id: fields[1].into(),
                priority,
                scope: fields[3].into(),
                severity: fields[4].into(),
                condition: fields[5].into(),
                tags: fields[6]
                    .split(',')
                    .map(str::trim)
                    .filter(|tag| !tag.is_empty())
                    .map(str::to_string)
                    .collect(),
                suppress: fields[7].into(),
                source: fields[8].into(),
            });
        } else if line.starts_with("MSG\t") {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 3 {
                return Err(invalid(format!(
                    "line {line_number}: MSG has {} fields, expected 3",
                    fields.len()
                )));
            }
            let pending = pending.take().ok_or_else(|| {
                invalid(format!("line {line_number}: MSG without preceding RULE"))
            })?;
            callback(Rule {
                id: pending.id,
                priority: pending.priority,
                scope: pending.scope,
                severity: pending.severity,
                condition: pending.condition,
                tags: pending.tags,
                suppress: pending.suppress,
                source: pending.source,
                title: fields[1].into(),
                message: fields[2].into(),
            })?;
            count += 1;
        }
    }
    if pending.is_some() {
        return Err(invalid("rulebook ended before final MSG"));
    }
    Ok(count)
}

pub fn verify(path: &Path) -> io::Result<Verification> {
    let sha256 = sha256_file(path)?;
    if sha256 != CANONICAL_SHA256 {
        return Err(invalid(format!(
            "canonical SHA-256 mismatch: expected {CANONICAL_SHA256}, got {sha256}"
        )));
    }

    let mut ids = HashSet::with_capacity(EXPECTED_RULES);
    let mut family_counts: HashMap<&'static str, usize> = HashMap::new();
    let mut sources = HashMap::new();
    let mut duplicate_ids = 0usize;
    let mut invalid_conditions = 0usize;
    let count = for_each_rule(path, |rule| {
        if !ids.insert(rule.id.clone()) {
            duplicate_ids += 1;
        }
        let family = family_for_id(&rule.id)
            .ok_or_else(|| invalid(format!("unknown rule family for {}", rule.id)))?;
        *family_counts.entry(family).or_default() += 1;
        *sources.entry(rule.source.clone()).or_default() += 1;
        if dsl::validate(&rule.condition).is_err() {
            invalid_conditions += 1;
        }
        Ok(())
    })?;

    if count != EXPECTED_RULES {
        return Err(invalid(format!(
            "rule count mismatch: expected {EXPECTED_RULES}, got {count}"
        )));
    }
    if duplicate_ids != 0 {
        return Err(invalid(format!("duplicate rule IDs: {duplicate_ids}")));
    }
    if invalid_conditions != 0 {
        return Err(invalid(format!(
            "syntactically invalid conditions: {invalid_conditions}"
        )));
    }
    for (family, expected) in FAMILIES {
        let actual = family_counts.get(family).copied().unwrap_or_default();
        if actual != *expected {
            return Err(invalid(format!(
                "family {family} mismatch: expected {expected}, got {actual}"
            )));
        }
    }
    if family_counts.len() != EXPECTED_FAMILIES {
        return Err(invalid(format!(
            "family count mismatch: expected {EXPECTED_FAMILIES}, got {}",
            family_counts.len()
        )));
    }
    Ok(Verification {
        sha256,
        rules: count,
        families: family_counts.len(),
        duplicate_ids,
        invalid_conditions,
        sources,
    })
}

pub fn evaluate(path: &Path, context: &Context, max_hits: usize) -> io::Result<Evaluation> {
    let mut triggered = Vec::new();
    let mut indeterminate = 0usize;
    let rules_evaluated = for_each_rule(path, |rule| {
        match dsl::evaluate(&rule.condition, context) {
            Truth::True => triggered.push(to_hit(rule)),
            Truth::Unknown => indeterminate += 1,
            Truth::False => {}
        }
        Ok(())
    })?;
    let rules_triggered = triggered.len();
    let hits = resolve_hits(triggered, max_hits);
    Ok(Evaluation {
        rules_evaluated,
        rules_triggered,
        rules_indeterminate: indeterminate,
        hits,
    })
}

fn resolve_hits(mut hits: Vec<RuleHit>, max_hits: usize) -> Vec<RuleHit> {
    let suppression_prefixes = hits
        .iter()
        .flat_map(|hit| hit.suppress.split([';', '|']))
        .filter_map(|value| {
            let prefix = value.trim().split(':').next()?.trim();
            (!prefix.is_empty()).then(|| prefix.to_string())
        })
        .collect::<HashSet<_>>();
    hits.retain(|hit| {
        !suppression_prefixes
            .iter()
            .any(|prefix| hit.id.starts_with(&format!("{prefix}-")))
            || !hit.suppress.is_empty()
    });
    hits.sort_by(compare_hits);

    let mut selected = Vec::with_capacity(max_hits.min(hits.len()));
    let mut seen = HashSet::new();
    let mut counters_by_scope: HashMap<String, usize> = HashMap::new();
    for hit in hits {
        let dedupe_key = (hit.channel, hit.scope.clone(), hit.title.clone());
        if !seen.insert(dedupe_key) {
            continue;
        }
        if hit.channel == Channel::CounterSignal {
            let count = counters_by_scope.entry(hit.scope.clone()).or_default();
            if *count >= 2 {
                continue;
            }
            *count += 1;
        }
        selected.push(hit);
        if selected.len() == max_hits {
            break;
        }
    }
    selected
}

fn compare_hits(left: &RuleHit, right: &RuleHit) -> Ordering {
    channel_rank(left.channel)
        .cmp(&channel_rank(right.channel))
        .then_with(|| severity_rank(&right.severity).cmp(&severity_rank(&left.severity)))
        .then_with(|| left.priority.cmp(&right.priority))
        .then_with(|| right.specificity.cmp(&left.specificity))
        .then_with(|| left.id.cmp(&right.id))
}

fn channel_rank(channel: Channel) -> u8 {
    match channel {
        Channel::DataQuality => 0,
        Channel::Primary => 1,
        Channel::Confirmation => 2,
        Channel::CounterSignal => 3,
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity.to_uppercase().as_str() {
        "CRITICAL" | "EXTREME" => 5,
        "RED" => 4,
        "ORANGE" => 3,
        "YELLOW" => 2,
        "GREEN" => 1,
        _ => 0,
    }
}

fn to_hit(rule: Rule) -> RuleHit {
    let tags_upper = rule.tags.join(",").to_uppercase();
    let channel = if rule.scope.eq_ignore_ascii_case("DATA")
        || tags_upper.contains("DATA_QUALITY")
        || tags_upper.contains("CADENCE")
    {
        Channel::DataQuality
    } else if tags_upper.contains("RECOVER")
        || tags_upper.contains("RELIEF")
        || tags_upper.contains("COUNTER")
    {
        Channel::CounterSignal
    } else if tags_upper.contains("CONFIRM") {
        Channel::Confirmation
    } else {
        Channel::Primary
    };
    let specificity = rule.condition.matches(" AND ").count()
        + rule.condition.matches(" OR ").count()
        + rule.condition.matches('(').count();
    RuleHit {
        id: rule.id,
        priority: rule.priority,
        scope: rule.scope,
        severity: rule.severity,
        channel,
        title: rule.title,
        message: rule.message,
        condition: rule.condition,
        tags: rule.tags,
        source: rule.source,
        suppress: rule.suppress,
        specificity,
    }
}

fn family_for_id(id: &str) -> Option<&'static str> {
    FAMILIES
        .iter()
        .filter_map(|(family, _)| {
            (id == *family || id.starts_with(&format!("{family}-"))).then_some(*family)
        })
        .max_by_key(|family| family.len())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn canonical_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("rulebook")
            .join("Market_Economy_Radar_Rulebook_v4_ULTRA.txt")
    }

    #[test]
    fn canonical_rulebook_has_exact_integrity_and_topology() {
        let verification = verify(&canonical_path()).unwrap();
        assert_eq!(verification.rules, EXPECTED_RULES);
        assert_eq!(verification.families, EXPECTED_FAMILIES);
        assert_eq!(verification.sources.get("v4"), Some(&8_031));
        assert_eq!(verification.sources.get("v3"), Some(&14_155));
        assert_eq!(verification.sources.get("v2"), Some(&5_308));
    }

    #[test]
    fn exact_fields_and_messages_are_preserved() {
        let mut first = None;
        for_each_rule(&canonical_path(), |rule| {
            if first.is_none() {
                first = Some(rule);
            }
            Ok(())
        })
        .unwrap();
        let first = first.unwrap();
        assert_eq!(first.id, "SINGLE-0001");
        assert_eq!(first.scope, "GLOBAL");
        assert_eq!(
            first.condition,
            "band(GROWTH)=GREEN AND dynamic(GROWTH)=IMPROVING_FAST"
        );
        assert!(first.message.contains("실물경기"));
    }

    #[test]
    fn empty_context_does_not_fabricate_hits() {
        let result = evaluate(&canonical_path(), &Context::default(), 64).unwrap();
        assert_eq!(result.rules_evaluated, EXPECTED_RULES);
        assert_eq!(result.rules_triggered, 0);
        assert!(result.hits.is_empty());
    }
}
