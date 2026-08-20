use std::collections::HashMap;

#[derive(Default)]
pub struct Context {
    pub values: HashMap<String, f64>,
    pub scores: HashMap<String, f64>,
}
impl Context {
    pub fn value(&self, k: &str) -> Option<f64> {
        self.values
            .get(k)
            .copied()
            .or_else(|| self.scores.get(k).copied())
    }
}

fn cmp(lhs: f64, op: &str, rhs: f64) -> bool {
    match op {
        ">" => lhs > rhs,
        ">=" => lhs >= rhs,
        "<" => lhs < rhs,
        "<=" => lhs <= rhs,
        "==" => (lhs - rhs).abs() < f64::EPSILON,
        "!=" => (lhs - rhs).abs() >= f64::EPSILON,
        _ => false,
    }
}

pub fn eval(expr: &str, ctx: &Context) -> bool {
    let e = expr.trim();
    if e.is_empty() {
        return false;
    }
    if let Some((a, b)) = split_top(e, " OR ") {
        return eval(a, ctx) || eval(b, ctx);
    }
    if let Some((a, b)) = split_top(e, " AND ") {
        return eval(a, ctx) && eval(b, ctx);
    }
    if e.starts_with('(') && e.ends_with(')') {
        return eval(&e[1..e.len() - 1], ctx);
    }
    for op in [">=", "<=", "!=", "==", ">", "<"] {
        if let Some((l, r)) = e.split_once(op) {
            if let (Some(a), Ok(b)) = (resolve(l.trim(), ctx), r.trim().parse::<f64>()) {
                return cmp(a, op, b);
            }
        }
    }
    false
}

fn resolve(s: &str, ctx: &Context) -> Option<f64> {
    if let Ok(x) = s.parse() {
        return Some(x);
    }
    for p in ["score(", "module(", "risk(", "metric("] {
        if s.starts_with(p) && s.ends_with(')') {
            return ctx.value(s[p.len()..s.len() - 1].trim_matches('"'));
        }
    }
    ctx.value(s)
}

fn split_top<'a>(s: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    let b = s.as_bytes();
    let n = needle.as_bytes();
    let mut i = 0;
    while i + n.len() <= b.len() {
        match b[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && &b[i..i + n.len()] == n {
            return Some((&s[..i], &s[i + n.len()..]));
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        let mut c = Context::default();
        c.values.insert("X".into(), 80.0);
        assert!(eval("X >= 75", &c));
        assert!(eval("X >= 75 AND X < 90", &c));
    }
}
