use crate::scoring;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Signal {
    pub current: Option<f64>,
    pub history: Vec<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct SourceState {
    pub age_days: Option<f64>,
    pub missing: bool,
    pub revised: bool,
    pub fresh: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Context {
    signals: HashMap<String, Signal>,
    booleans: HashMap<String, bool>,
    texts: HashMap<String, String>,
    sources: HashMap<String, SourceState>,
}

impl Context {
    pub fn insert_signal(&mut self, name: impl AsRef<str>, signal: Signal) {
        self.signals.insert(normalize_name(name.as_ref()), signal);
    }

    pub fn insert_number(&mut self, name: impl AsRef<str>, value: f64) {
        self.insert_signal(
            name,
            Signal {
                current: Some(value),
                history: Vec::new(),
            },
        );
    }

    pub fn insert_bool(&mut self, name: impl AsRef<str>, value: bool) {
        self.booleans.insert(normalize_name(name.as_ref()), value);
    }

    pub fn insert_text(&mut self, name: impl AsRef<str>, value: impl Into<String>) {
        self.texts
            .insert(normalize_name(name.as_ref()), value.into().to_uppercase());
    }

    pub fn insert_source(&mut self, name: impl AsRef<str>, state: SourceState) {
        self.sources.insert(normalize_name(name.as_ref()), state);
    }

    pub fn signal(&self, name: &str) -> Option<&Signal> {
        self.signals.get(&normalize_name(name))
    }

    pub fn value(&self, name: &str) -> Option<f64> {
        self.signal(name).and_then(|signal| signal.current)
    }

    pub fn known_signal_count(&self) -> usize {
        self.signals
            .values()
            .filter(|signal| signal.current.is_some())
            .count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Truth {
    True,
    False,
    Unknown,
}

impl Truth {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Unknown,
        }
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Unknown,
        }
    }

    fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
enum Value {
    Number(f64),
    Bool(bool),
    Text(String),
    Unknown,
}

fn normalize_name(name: &str) -> String {
    name.trim()
        .trim_matches(['"', '\'', '[', ']'])
        .to_uppercase()
}

pub fn validate(expression: &str) -> Result<(), String> {
    let mut stack = Vec::new();
    let mut quote = None;
    for (offset, ch) in expression.char_indices() {
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match ch {
            '(' | '[' => stack.push((ch, offset)),
            ')' => match stack.pop() {
                Some(('(', _)) => {}
                _ => return Err(format!("unmatched ')' at byte {offset}")),
            },
            ']' => match stack.pop() {
                Some(('[', _)) => {}
                _ => return Err(format!("unmatched ']' at byte {offset}")),
            },
            _ => {}
        }
    }
    if let Some((delimiter, offset)) = stack.pop() {
        return Err(format!("unclosed '{delimiter}' at byte {offset}"));
    }
    if quote.is_some() {
        return Err("unclosed quote".into());
    }
    if expression.trim().is_empty() {
        return Err("empty expression".into());
    }
    Ok(())
}

pub fn evaluate(expression: &str, context: &Context) -> Truth {
    if validate(expression).is_err() {
        return Truth::Unknown;
    }
    let locals = derive_locals(expression, context);
    eval_boolean(expression.trim(), context, &locals)
}

fn derive_locals(expression: &str, context: &Context) -> HashMap<String, Value> {
    let mut locals = HashMap::new();
    let upper = expression.to_uppercase();
    for macro_name in [
        "V4MKT3", "MKT5", "MKT4", "MKT3", "V4MP", "V4MT", "V4PATH4", "V4PATH3", "PATH5", "PATH4",
        "PATH3", "HPAIR", "EDGEH", "EDGE",
    ] {
        let needle = format!("{macro_name}(");
        let Some(start) = upper.find(&needle) else {
            continue;
        };
        let open = start + macro_name.len();
        let Some(close) = matching_close(expression, open, '(', ')') else {
            continue;
        };
        let arguments = &expression[open + 1..close];
        let nodes = tokenize_nodes(arguments)
            .into_iter()
            .filter(|node| context.value(node).is_some())
            .collect::<Vec<_>>();
        if nodes.is_empty() {
            continue;
        }
        let scores = nodes
            .iter()
            .filter_map(|node| context.value(node))
            .collect::<Vec<_>>();
        if let Some(minimum) = scores.iter().copied().reduce(f64::min) {
            locals.insert("MINMOD".into(), Value::Number(minimum));
            locals.insert("MIN_SCORE".into(), Value::Number(minimum));
        }
        if let Some(maximum) = scores.iter().copied().reduce(f64::max) {
            locals.insert("MAXMOD".into(), Value::Number(maximum));
            locals.insert("MAX_SCORE".into(), Value::Number(maximum));
        }
        let deltas = nodes
            .iter()
            .filter_map(|node| delta(context, node, 7))
            .collect::<Vec<_>>();
        if deltas.len() == nodes.len() {
            if let Some(minimum) = deltas.iter().copied().reduce(f64::min) {
                locals.insert("MINMODDELTA7".into(), Value::Number(minimum));
            }
            if let Some(maximum) = deltas.iter().copied().reduce(f64::max) {
                locals.insert("MAXMODDELTA7".into(), Value::Number(maximum));
            }
        }
        break;
    }
    locals
}

fn eval_boolean(expression: &str, context: &Context, locals: &HashMap<String, Value>) -> Truth {
    let expression = strip_outer(expression.trim());
    if let Some((left, right)) = split_top(expression, " OR ") {
        return eval_boolean(left, context, locals).or(eval_boolean(right, context, locals));
    }
    if let Some((left, right)) = split_top(expression, " AND ") {
        return eval_boolean(left, context, locals).and(eval_boolean(right, context, locals));
    }
    if let Some(rest) = expression.strip_prefix("NOT ") {
        return eval_boolean(rest, context, locals).not();
    }
    eval_atom(expression, context, locals)
}

fn eval_atom(expression: &str, context: &Context, locals: &HashMap<String, Value>) -> Truth {
    let expression = strip_outer(expression.trim());
    if let Some((value, bounds)) = split_top(expression, " BETWEEN ") {
        if let Some((low, high)) = split_top(bounds, " AND ") {
            return match (
                eval_value(value, context, locals),
                eval_value(low, context, locals),
                eval_value(high, context, locals),
            ) {
                (Value::Number(value), Value::Number(low), Value::Number(high)) => {
                    Truth::from(value >= low && value <= high)
                }
                _ => Truth::Unknown,
            };
        }
    }
    if contains_top_word(expression, " BEFORE ")
        || contains_top_word(expression, " WITHIN ")
        || contains_top_word(expression, " AGAIN ")
    {
        return context
            .booleans
            .get(&normalize_name(expression))
            .copied()
            .map(Truth::from)
            .unwrap_or(Truth::Unknown);
    }
    for operator in [">=", "<=", "!=", "==", "=", ">", "<"] {
        if let Some((left, right)) = split_top(expression, operator) {
            return compare(
                eval_value(left, context, locals),
                operator,
                eval_value(right, context, locals),
            );
        }
    }
    match eval_value(expression, context, locals) {
        Value::Bool(value) => Truth::from(value),
        Value::Number(value) => Truth::from(value != 0.0),
        _ => Truth::Unknown,
    }
}

impl From<bool> for Truth {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

fn compare(left: Value, operator: &str, right: Value) -> Truth {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Truth::from(match operator {
            ">=" => left >= right,
            "<=" => left <= right,
            ">" => left > right,
            "<" => left < right,
            "=" | "==" => (left - right).abs() <= f64::EPSILON,
            "!=" => (left - right).abs() > f64::EPSILON,
            _ => false,
        }),
        (Value::Bool(left), Value::Bool(right)) => Truth::from(match operator {
            "=" | "==" => left == right,
            "!=" => left != right,
            _ => false,
        }),
        (Value::Text(left), Value::Text(right)) => Truth::from(match operator {
            "=" | "==" => left.eq_ignore_ascii_case(&right),
            "!=" => !left.eq_ignore_ascii_case(&right),
            _ => false,
        }),
        _ => Truth::Unknown,
    }
}

fn eval_value(expression: &str, context: &Context, locals: &HashMap<String, Value>) -> Value {
    let expression = expression.trim();
    let normalized = normalize_name(expression);
    if let Some(value) = locals.get(&normalized) {
        return value.clone();
    }
    if let Some(value) = context.value(&normalized) {
        return Value::Number(value);
    }
    if let Some(value) = context.booleans.get(&normalized) {
        return Value::Bool(*value);
    }
    if let Some(value) = context.texts.get(&normalized) {
        return Value::Text(value.clone());
    }
    if normalized == "TRUE" {
        return Value::Bool(true);
    }
    if normalized == "FALSE" {
        return Value::Bool(false);
    }
    if let Ok(value) = expression.parse::<f64>() {
        return Value::Number(value);
    }
    if normalized.starts_with('P') && normalized[1..].chars().all(|ch| ch.is_ascii_digit()) {
        if let Ok(value) = normalized[1..].parse::<f64>() {
            return Value::Number(value);
        }
    }
    if let Some(duration) = normalized.strip_suffix('D') {
        if let Ok(value) = duration.parse::<f64>() {
            return Value::Number(value);
        }
    }
    if let Some(open) = expression.find('(') {
        if let Some(close) = matching_close(expression, open, '(', ')') {
            let name = normalize_name(&expression[..open]);
            let raw_arguments = &expression[open + 1..close];
            let arguments = split_arguments(raw_arguments);
            let exact_key = format!("{}:{}", name, normalize_name(raw_arguments));
            if let Some(value) = context.value(&exact_key) {
                return Value::Number(value);
            }
            if let Some(value) = context.booleans.get(&exact_key) {
                return Value::Bool(*value);
            }
            if let Some(value) = context.texts.get(&exact_key) {
                return Value::Text(value.clone());
            }
            return eval_function(&name, &arguments, context, locals);
        }
    }
    if is_literal(&normalized) {
        Value::Text(normalized)
    } else {
        Value::Unknown
    }
}

fn eval_function(
    name: &str,
    arguments: &[String],
    context: &Context,
    locals: &HashMap<String, Value>,
) -> Value {
    let argument = |index: usize| {
        arguments
            .get(index)
            .map(|value| value.as_str())
            .unwrap_or("")
    };
    match name {
        "SCORE"
        | "RISK"
        | "METRIC"
        | "MOD"
        | "METRIC_RISK"
        | "MKT_STRESS"
        | "MKT_VULN"
        | "HORIZON"
        | "CONTRIB"
        | "GROUP_RISK"
        | "STAGE_CONF"
        | "CONFIRM"
        | "CONTRADICT"
        | "DISPERSION"
        | "REVISION_RISK"
        | "PEAK"
        | "RECOVERY_DAYS"
        | "SURPRISE_VOL"
        | "SOURCE_CORR"
        | "SOURCE_CONFIRM"
        | "ONE_SOURCE_DOMINANCE"
        | "SOURCE_FRESHNESS_GAP"
        | "SOURCE_DISAGREE"
        | "CROSS_SOURCE_CONFIRM" => context
            .value(&format!("{}:{}", name, normalize_name(argument(0))))
            .or_else(|| context.value(argument(0)))
            .map(Value::Number)
            .unwrap_or(Value::Unknown),
        "HIST_SIM"
        | "CLUSTER_BREADTH"
        | "COUNT_HIGH"
        | "NEGATIVE_SURPRISE_STREAK"
        | "POSITIVE_SURPRISE_STREAK"
        | "DYNAMIC_WEIGHT" => context
            .value(&format!(
                "{}:{}",
                name,
                normalize_name(&arguments.join(","))
            ))
            .or_else(|| match name {
                "NEGATIVE_SURPRISE_STREAK" => surprise_streak(context, argument(0), false),
                "POSITIVE_SURPRISE_STREAK" => surprise_streak(context, argument(0), true),
                _ => None,
            })
            .map(Value::Number)
            .unwrap_or(Value::Unknown),
        "DELTA7" | "MODDELTA7" | "CONTRIB_DELTA7" | "METRIC_DELTA" => {
            delta(context, argument(0), 7)
                .map(Value::Number)
                .unwrap_or(Value::Unknown)
        }
        "DELTA20" => delta(context, argument(0), 20)
            .map(Value::Number)
            .unwrap_or(Value::Unknown),
        "ACCELERATION" => acceleration(context, argument(0))
            .map(Value::Number)
            .unwrap_or(Value::Unknown),
        "ABS" => match eval_value(argument(0), context, locals) {
            Value::Number(value) => Value::Number(value.abs()),
            _ => Value::Unknown,
        },
        "BAND" | "MODBAND" => context
            .value(argument(0))
            .map(|value| Value::Text(band(value).into()))
            .unwrap_or(Value::Unknown),
        "DYNAMIC" | "MODDYN" | "HDYNAMIC" => dynamic(context, argument(0))
            .map(|value| Value::Text(value.into()))
            .unwrap_or(Value::Unknown),
        "SURPRISE" | "REVISION" | "STAGE_ROLE" => context
            .texts
            .get(&format!("{}:{}", name, normalize_name(argument(0))))
            .cloned()
            .map(Value::Text)
            .unwrap_or(Value::Unknown),
        "PERSIST5" | "MODPERSIST5" | "METRIC_PERSIST" => persist(context, argument(0), 5)
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "SHOCK" => delta(context, argument(0), 7)
            .map(|value| Value::Bool(value >= 10.0))
            .unwrap_or(Value::Unknown),
        "CROSS_UP" => cross_up(context, argument(0), argument(1))
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "CORR" | "STRESS_CORR" | "CORR_SHIFT" | "CORR_VOL" | "TAIL_DEP" => {
            let left = context.signal(argument(0));
            let right = context.signal(argument(1));
            match (left, right) {
                (Some(left), Some(right)) => scoring::correlation(&left.history, &right.history)
                    .map(Value::Number)
                    .unwrap_or(Value::Unknown),
                _ => Value::Unknown,
            }
        }
        "SOURCE_AGE" => context
            .sources
            .get(&normalize_name(argument(0)))
            .and_then(|state| state.age_days)
            .map(Value::Number)
            .unwrap_or(Value::Unknown),
        "SOURCE_MISSING" => context
            .sources
            .get(&normalize_name(argument(0)))
            .map(|state| Value::Bool(state.missing))
            .unwrap_or(Value::Unknown),
        "SOURCE_REVISION" => context
            .sources
            .get(&normalize_name(argument(0)))
            .map(|state| Value::Bool(state.revised))
            .unwrap_or(Value::Unknown),
        "SOURCE_OK" => context
            .sources
            .get(&normalize_name(argument(0)))
            .map(|state| Value::Bool(state.fresh && !state.missing))
            .unwrap_or(Value::Unknown),
        "IMPROVING" => delta(context, argument(0), 7)
            .map(|value| Value::Bool(value <= -3.0))
            .unwrap_or(Value::Unknown),
        "PREVIOUS_RED" => previous_red(context, argument(0))
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "RECOVERED" => recovered(context, argument(0))
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "RECOVERY_FAILED" => recovery_failed(context, argument(0))
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "NEW_HIGH_RISK" => new_high_risk(context, argument(0))
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "TOP3_CONTRIBUTOR" | "MARKET_BREADTH" | "POLICY_BUFFER" | "ORDERED_CROSS"
        | "ORDERED_SHOCK" | "CHAIN_BREAK_AT" | "CHAIN_BREAK_BEFORE" => context
            .booleans
            .get(&format!(
                "{}:{}",
                name,
                normalize_name(&arguments.join(","))
            ))
            .copied()
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
        "EDGE" | "EDGEH" | "HPAIR" | "PATH3" | "PATH4" | "PATH5" | "V4PATH3" | "V4PATH4"
        | "MKT3" | "MKT4" | "MKT5" | "V4MKT3" | "V4MP" | "V4MT" | "SHOCKVULN" => {
            structural(arguments, context)
        }
        _ => context
            .booleans
            .get(&format!(
                "{}:{}",
                name,
                normalize_name(&arguments.join(","))
            ))
            .copied()
            .map(Value::Bool)
            .unwrap_or(Value::Unknown),
    }
}

fn structural(arguments: &[String], context: &Context) -> Value {
    let nodes = arguments
        .iter()
        .flat_map(|argument| tokenize_nodes(argument))
        .filter(|node| !is_scope(node))
        .collect::<Vec<_>>();
    if nodes.is_empty() {
        return Value::Unknown;
    }
    if nodes.iter().all(|node| context.value(node).is_some()) {
        Value::Bool(true)
    } else {
        Value::Unknown
    }
}

fn delta(context: &Context, name: &str, period: usize) -> Option<f64> {
    let signal = context.signal(name)?;
    let current = signal.current?;
    let comparison = signal.history.len().checked_sub(period + 1)?;
    Some(current - signal.history[comparison])
}

fn acceleration(context: &Context, name: &str) -> Option<f64> {
    let signal = context.signal(name)?;
    if signal.history.len() < 15 {
        return None;
    }
    let len = signal.history.len();
    let recent = signal.history[len - 1] - signal.history[len - 8];
    let previous = signal.history[len - 8] - signal.history[len - 15];
    Some(recent - previous)
}

fn persist(context: &Context, name: &str, periods: usize) -> Option<bool> {
    let signal = context.signal(name)?;
    if signal.history.len() < periods {
        return None;
    }
    let current = signal.current?;
    let current_band = band(current);
    Some(
        signal.history[signal.history.len() - periods..]
            .iter()
            .all(|value| band(*value) == current_band),
    )
}

fn cross_up(context: &Context, name: &str, threshold: &str) -> Option<bool> {
    let threshold = threshold.parse::<f64>().ok()?;
    let signal = context.signal(name)?;
    let current = signal.current?;
    let previous = signal.history.last().copied()?;
    Some(previous < threshold && current >= threshold)
}

fn surprise_streak(context: &Context, name: &str, positive: bool) -> Option<f64> {
    let signal = context.signal(&format!("SURPRISE:{name}"))?;
    let mut values = signal.history.clone();
    if let Some(current) = signal.current {
        values.push(current);
    }
    Some(
        values
            .iter()
            .rev()
            .take_while(|value| {
                if positive {
                    **value > 0.0
                } else {
                    **value < 0.0
                }
            })
            .count() as f64,
    )
}

fn previous_red(context: &Context, name: &str) -> Option<bool> {
    context
        .signal(name)?
        .history
        .last()
        .map(|value| *value >= 75.0)
}

fn recovered(context: &Context, name: &str) -> Option<bool> {
    let signal = context.signal(name)?;
    let current = signal.current?;
    Some(
        current < 55.0
            && signal
                .history
                .iter()
                .rev()
                .take(20)
                .any(|value| *value >= 75.0),
    )
}

fn recovery_failed(context: &Context, name: &str) -> Option<bool> {
    let signal = context.signal(name)?;
    let current = signal.current?;
    let recent = signal
        .history
        .iter()
        .rev()
        .take(20)
        .copied()
        .collect::<Vec<_>>();
    Some(
        current >= 65.0
            && recent.iter().any(|value| *value < 55.0)
            && recent.iter().any(|value| *value >= 75.0),
    )
}

fn new_high_risk(context: &Context, name: &str) -> Option<bool> {
    let signal = context.signal(name)?;
    let current = signal.current?;
    let previous_high = signal.history.iter().copied().reduce(f64::max)?;
    Some(current > previous_high)
}

fn band(value: f64) -> &'static str {
    if value >= 75.0 {
        "RED"
    } else if value >= 55.0 {
        "ORANGE"
    } else if value >= 35.0 {
        "YELLOW"
    } else {
        "GREEN"
    }
}

fn dynamic(context: &Context, name: &str) -> Option<&'static str> {
    let change = delta(context, name, 7)?;
    Some(if change >= 10.0 {
        "WORSENING_FAST"
    } else if change >= 3.0 {
        "WORSENING"
    } else if change <= -10.0 {
        "IMPROVING_FAST"
    } else if change <= -3.0 {
        "IMPROVING"
    } else {
        "FLAT"
    })
}

fn is_scope(value: &str) -> bool {
    matches!(
        normalize_name(value).as_str(),
        "GLOBAL" | "US_EQUITY" | "KOREA_EQUITY" | "CRYPTO" | "DATA" | "ENGINE"
    )
}

fn is_literal(value: &str) -> bool {
    matches!(
        value,
        "ACCELERATING"
            | "ACTIVE"
            | "ADEQUATE"
            | "BANKING"
            | "BUILDING"
            | "CALM"
            | "CHINA_SPILLOVER"
            | "CONFIRMING"
            | "CONSUMER"
            | "CORP_HEALTH"
            | "COUNTER"
            | "CREDIT"
            | "DOWN"
            | "ELEVATED"
            | "EXTREME"
            | "FINCOND"
            | "FLAT"
            | "FUNDING"
            | "GREEN"
            | "GROWTH"
            | "HIGH"
            | "HOUSING"
            | "IMPROVING"
            | "IMPROVING_FAST"
            | "INFO"
            | "INFLATION"
            | "KOREA_MACRO"
            | "LAGGING"
            | "LABOR"
            | "LEADING"
            | "LEVERAGE"
            | "LIQUIDITY"
            | "LOW"
            | "MODERATE"
            | "NEG_TO_POS"
            | "NEGATIVE"
            | "NEGATIVE_EXTREME"
            | "NEUTRAL"
            | "NORMAL"
            | "OFFSET"
            | "ORANGE"
            | "POS_TO_NEG"
            | "POSITIVE"
            | "POSITIVE_EXTREME"
            | "RATES"
            | "RED"
            | "STRONG"
            | "THIN"
            | "TREASURY"
            | "UP"
            | "USD"
            | "VOLATILITY"
            | "WEAK"
            | "WORSENING"
            | "WORSENING_FAST"
            | "YELLOW"
    )
}

fn tokenize_nodes(input: &str) -> Vec<&str> {
    input
        .split([',', ';'])
        .flat_map(|value| value.split("->"))
        .map(str::trim)
        .filter(|value| {
            !value.is_empty()
                && !value.contains('=')
                && !value.ends_with('d')
                && !value.chars().all(|ch| ch.is_ascii_digit())
        })
        .collect()
}

fn split_arguments(input: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' | ';' if depth == 0 => {
                values.push(input[start..index].trim().to_string());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    values.push(input[start..].trim().to_string());
    values
}

fn strip_outer(mut expression: &str) -> &str {
    loop {
        let trimmed = expression.trim();
        if !trimmed.starts_with('(') {
            return trimmed;
        }
        let Some(close) = matching_close(trimmed, 0, '(', ')') else {
            return trimmed;
        };
        if close != trimmed.len() - 1 {
            return trimmed;
        }
        expression = &trimmed[1..close];
    }
}

fn matching_close(input: &str, open: usize, opener: char, closer: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    for (index, ch) in input.char_indices().filter(|(index, _)| *index >= open) {
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            continue;
        }
        if quote.is_some() {
            continue;
        }
        if ch == opener {
            depth += 1;
        } else if ch == closer {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn contains_top_word(input: &str, needle: &str) -> bool {
    split_top(input, needle).is_some()
}

fn split_top<'a>(input: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let bytes = input.as_bytes();
    let needle_bytes = needle.as_bytes();
    let mut round = 0i32;
    let mut square = 0i32;
    let mut quote = None;
    let mut between_pending = false;
    let mut index = 0;
    while index + needle_bytes.len() <= bytes.len() {
        let ch = input[index..].chars().next()?;
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
            index += ch.len_utf8();
            continue;
        }
        if quote.is_none() {
            match ch {
                '(' => round += 1,
                ')' => round -= 1,
                '[' => square += 1,
                ']' => square -= 1,
                _ => {}
            }
            if round == 0 && square == 0 {
                if input[index..].starts_with(" BETWEEN ") {
                    between_pending = true;
                }
                if &bytes[index..index + needle_bytes.len()] == needle_bytes {
                    if needle == " AND " && between_pending {
                        between_pending = false;
                    } else {
                        return Some((&input[..index], &input[index + needle.len()..]));
                    }
                }
            }
        }
        index += ch.len_utf8();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        let mut context = Context::default();
        context.insert_signal(
            "GROWTH",
            Signal {
                current: Some(80.0),
                history: (0..30).map(|value| 40.0 + value as f64).collect(),
            },
        );
        context.insert_number("LABOR", 70.0);
        context.insert_number("VALUATION", 75.0);
        context.insert_number("BUSINESS_DEBT", 72.0);
        context
    }

    #[test]
    fn numeric_boolean_and_band_conditions_work() {
        let context = context();
        assert_eq!(
            evaluate("score(GROWTH)>=65 AND score(LABOR)>=65", &context),
            Truth::True
        );
        assert_eq!(evaluate("band(GROWTH)=RED", &context), Truth::True);
        assert_eq!(
            evaluate("score(GROWTH) BETWEEN 68 AND 84", &context),
            Truth::True
        );
    }

    #[test]
    fn missing_values_are_unknown_not_fabricated() {
        let context = Context::default();
        assert_eq!(evaluate("score(GROWTH)>=65", &context), Truth::Unknown);
        assert_eq!(evaluate("PRICE_TREND=UP", &context), Truth::Unknown);
    }

    #[test]
    fn version_number_does_not_change_macro_arity() {
        let context = context();
        assert_eq!(
            evaluate(
                "V4MP(VALUATION,BUSINESS_DEBT) AND MOD(VALUATION)>=65 AND MOD(BUSINESS_DEBT)>=65",
                &context
            ),
            Truth::True
        );
    }
}
