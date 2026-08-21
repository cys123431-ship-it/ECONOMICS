from pathlib import Path
import re


def replace_exact(text: str, old: str, new: str, label: str, count: int = 1) -> str:
    actual = text.count(old)
    if actual != count:
        raise SystemExit(f"{label}: expected {count} occurrence(s), found {actual}")
    return text.replace(old, new)


collectors_path = Path("src/collectors.rs")
collectors = collectors_path.read_text(encoding="utf-8")
collectors = replace_exact(
    collectors,
    "use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, SecondsFormat, Utc, Weekday};",
    "use chrono::{Datelike, Duration as ChronoDuration, FixedOffset, NaiveDate, SecondsFormat, Utc, Weekday};",
    "chrono FixedOffset import",
)
collectors = replace_exact(
    collectors,
    '.user_agent("ECONOMICS-Radar/0.3")',
    '.user_agent(concat!("ECONOMICS-Radar/", env!("CARGO_PKG_VERSION")))',
    "user agent version",
)
collectors = replace_exact(
    collectors,
    '    "EQTA",\n',
    '',
    "discontinued EQTA removal",
)
collectors = replace_exact(
    collectors,
    "// FRED's series/vintagedates endpoint accepts at most 1,000 rows per page.\n// A larger value makes every ALFRED request fail with HTTP 400.\n",
    "// Keep vintage-date pages conservative even though FRED permits larger limits.\n// Smaller pages reduce payload size and make transient failures cheaper to retry.\n",
    "FRED vintage page comment",
)
collectors = replace_exact(
    collectors,
    "    let end = today - ChronoDuration::days(2);",
    "    let end = today;",
    "KRX artificial two-day lag",
)
collectors = replace_exact(
    collectors,
    "    let today = Utc::now().date_naive();",
    "    let kst = FixedOffset::east_opt(9 * 60 * 60).expect(\"KST offset is valid\");\n    let today = Utc::now().with_timezone(&kst).date_naive();",
    "KRX KST date",
)

old_response_block = '''            Ok(response) => {
                let status = response.status();
                let retryable = status.is_server_error() || status.as_u16() == 429;
                let error = response
                    .error_for_status()
                    .expect_err("non-success FRED response must have an HTTP error");
                if !retryable || attempt + 1 == FRED_REQUEST_ATTEMPTS {
                    return Err(redact_url_error(error.to_string(), key));
                }
            }
'''
new_response_block = '''            Ok(response) => {
                let status = response.status();
                let retryable = status.is_server_error() || status.as_u16() == 429;
                let response_url = response.url().to_string();
                let body = response.text().unwrap_or_default();
                let detail = serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("error_message")
                            .or_else(|| value.get("message"))
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .unwrap_or_else(|| body.chars().take(240).collect());
                if !retryable || attempt + 1 == FRED_REQUEST_ATTEMPTS {
                    let suffix = if detail.trim().is_empty() {
                        String::new()
                    } else {
                        format!(": {detail}")
                    };
                    return Err(redact_url_error(
                        format!("HTTP status {status} for url ({response_url}){suffix}"),
                        key,
                    ));
                }
            }
'''
collectors = replace_exact(
    collectors,
    old_response_block,
    new_response_block,
    "FRED error body reporting",
)
collectors_path.write_text(collectors, encoding="utf-8")

js_path = Path("src/dashboard.js")
js = js_path.read_text(encoding="utf-8")
js = replace_exact(
    js,
    "    item.append(el('span', 'ticker-name', indicator.label), el('span', `ticker-change ${direction(indicator)}`, formatChange(indicator)));",
    "    const freshness = indicator.freshness || 'UNKNOWN';\n    const asOf = indicator.observed_at ? String(indicator.observed_at).slice(5, 10) : 'NO DATE';\n    item.append(el('span', 'ticker-name', `${indicator.label} · ${freshness} · ${asOf}`), el('span', `ticker-change ${direction(indicator)}`, formatChange(indicator)));",
    "ticker freshness",
)
js = replace_exact(
    js,
    "    header.append(el('span', 'symbol', indicator.symbol), el('span', 'source', indicator.source || 'NO SOURCE'));",
    "    header.append(el('span', 'symbol', indicator.symbol), el('span', 'source', `${indicator.source || 'NO SOURCE'} · ${indicator.freshness || 'UNKNOWN'}`));",
    "quote freshness",
)
js = replace_exact(
    js,
    "    row.append(sparkCell, el('td', 'indicator-label', indicator.observed_at ? String(indicator.observed_at).slice(0, 10) : '—'));",
    "    const basis = indicator.observed_at ? `${String(indicator.observed_at).slice(0, 10)} · ${indicator.freshness || 'UNKNOWN'}` : '—';\n    row.append(sparkCell, el('td', 'indicator-label', basis));",
    "table freshness",
)
js_path.write_text(js, encoding="utf-8")

print("v0.4.3 source patch applied")
