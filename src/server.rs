use crate::{dashboard, db::Db, refresh::RefreshControl};
use chrono::{Datelike, FixedOffset, NaiveDate, Timelike, Utc, Weekday};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'";
const DASHBOARD_HTML: &str = include_str!("dashboard.html");
const DASHBOARD_CSS: &str = include_str!("dashboard.css");
const DASHBOARD_JS: &str = include_str!("dashboard.js");
const TICKER_KEYS: &[&str] = &["usdkrw", "btc", "sp500", "nasdaq", "dow", "kospi", "kosdaq"];

pub fn serve_with_refresh(
    host: &str,
    db_path: PathBuf,
    refresh: RefreshControl,
    on_ready: impl FnOnce(),
) -> Result<(), Box<dyn std::error::Error>> {
    serve_internal(host, db_path, Some(refresh), on_ready)
}

fn serve_internal(
    host: &str,
    db_path: PathBuf,
    refresh: Option<RefreshControl>,
    on_ready: impl FnOnce(),
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(host)?;
    println!("ECONOMICS Radar: http://{host}");
    on_ready();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle(stream, &db_path, refresh.as_ref()) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn decorate_ticker_freshness(dashboard: &mut Value) {
    let Some(indicators) = dashboard
        .get_mut("indicators")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    for indicator in indicators {
        let Some(key) = indicator.get("key").and_then(Value::as_str) else {
            continue;
        };
        if !TICKER_KEYS.contains(&key) {
            continue;
        }
        let label = indicator
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(key)
            .to_string();
        let mut freshness = indicator
            .get("freshness")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string();
        let date = indicator
            .get("observed_at")
            .and_then(Value::as_str)
            .and_then(|value| value.get(..10))
            .unwrap_or("NO DATE")
            .to_string();
        let source = indicator
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if source == "KRX" {
            let kst = FixedOffset::east_opt(9 * 60 * 60).expect("KST offset is valid");
            let now = Utc::now().with_timezone(&kst);
            let today = now.date_naive();
            let observed = NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok();
            if observed == Some(today) {
                freshness = "KRX EOD TODAY".into();
            } else if observed.is_some_and(|value| value < today)
                && !matches!(now.weekday(), Weekday::Sat | Weekday::Sun)
            {
                let minute = now.hour() * 60 + now.minute();
                freshness = if minute < 16 * 60 {
                    "KRX EOD(오늘 16:00 이후)".into()
                } else if minute < 18 * 60 + 15 {
                    "KRX EOD(당일 공개 확인 중)".into()
                } else {
                    "KRX EOD(최신 공개 종가)".into()
                };
            }
        }
        if let Some(object) = indicator.as_object_mut() {
            object.insert(
                "label".into(),
                Value::String(format!("{label} · {freshness} · {date}")),
            );
        }
    }
}

fn dashboard_response(db: &Db) -> Result<String, Box<dyn std::error::Error>> {
    let snapshot = db
        .latest_snapshot_json()?
        .and_then(|payload| serde_json::from_str::<Value>(&payload).ok())
        .unwrap_or_else(|| json!({"status":"no_snapshot"}));
    let mut dashboard = serde_json::to_value(dashboard::build(db)?)?;
    decorate_ticker_freshness(&mut dashboard);
    Ok(serde_json::to_string(&json!({
        "snapshot": snapshot,
        "dashboard": dashboard,
    }))?)
}

fn handle(
    mut stream: TcpStream,
    db_path: &Path,
    refresh: Option<&RefreshControl>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0u8; 8192];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    if method == "POST" && path == "/api/refresh" {
        let Some(refresh) = refresh else {
            return respond(
                &mut stream,
                "503 Service Unavailable",
                "application/json; charset=utf-8",
                r#"{"accepted":false,"error":"automatic refresh is disabled"}"#,
            );
        };
        let accepted = refresh.request_full();
        return respond(
            &mut stream,
            if accepted {
                "202 Accepted"
            } else {
                "409 Conflict"
            },
            "application/json; charset=utf-8",
            if accepted {
                r#"{"accepted":true}"#
            } else {
                r#"{"accepted":false,"error":"refresh is already running or queued"}"#
            },
        );
    }
    if method != "GET" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "application/json; charset=utf-8",
            r#"{"error":"method not allowed"}"#,
        );
    }
    match path {
        "/api/snapshot" => {
            let db = Db::open(db_path)?;
            let body = db
                .latest_snapshot_json()?
                .unwrap_or_else(|| r#"{"status":"no_snapshot"}"#.into());
            respond(
                &mut stream,
                "200 OK",
                "application/json; charset=utf-8",
                &body,
            )
        }
        "/api/dashboard" => {
            let db = Db::open(db_path)?;
            let body = dashboard_response(&db)?;
            respond(
                &mut stream,
                "200 OK",
                "application/json; charset=utf-8",
                &body,
            )
        }
        "/api/refresh-status" => {
            let body = refresh.map(RefreshControl::status_json).unwrap_or_else(|| {
                r#"{"running":false,"errors":["automatic refresh is disabled"]}"#.into()
            });
            respond(
                &mut stream,
                "200 OK",
                "application/json; charset=utf-8",
                &body,
            )
        }
        "/health" => {
            let body = format!(
                r#"{{"status":"ok","version":"{}"}}"#,
                env!("CARGO_PKG_VERSION")
            );
            respond(
                &mut stream,
                "200 OK",
                "application/json; charset=utf-8",
                &body,
            )
        }
        "/app.js" => respond(
            &mut stream,
            "200 OK",
            "application/javascript; charset=utf-8",
            DASHBOARD_JS,
        ),
        "/app.css" => respond(
            &mut stream,
            "200 OK",
            "text/css; charset=utf-8",
            DASHBOARD_CSS,
        ),
        "/" => respond(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            DASHBOARD_HTML,
        ),
        _ => respond(
            &mut stream,
            "404 Not Found",
            "application/json; charset=utf-8",
            r#"{"error":"not found"}"#,
        ),
    }
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: {CONTENT_SECURITY_POLICY}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_uses_a_same_origin_script_allowed_by_csp() {
        assert!(DASHBOARD_HTML.contains("src=\"/app.js\""));
        assert!(DASHBOARD_HTML.contains("defer"));
        assert!(!DASHBOARD_HTML.contains("<script>"));
        assert!(DASHBOARD_HTML.contains("href=\"/app.css\""));
        assert!(!DASHBOARD_HTML.contains("<style>"));
        assert!(CONTENT_SECURITY_POLICY.contains("script-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("style-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("connect-src 'self'"));
        assert!(DASHBOARD_JS.contains("fetch('/api/dashboard'"));
        assert!(DASHBOARD_JS.contains("대시보드 API 실패"));
        assert!(DASHBOARD_JS.contains("renderMarket"));
        assert!(DASHBOARD_JS.contains("/api/refresh-status"));
        assert!(DASHBOARD_JS.contains("method: 'POST'"));
        assert!(DASHBOARD_HTML.contains("data-tab=\"us\""));
        assert!(DASHBOARD_HTML.contains("data-tab=\"korea\""));
        assert!(DASHBOARD_HTML.contains("data-tab=\"crypto\""));
        assert!(DASHBOARD_CSS.contains(".dial"));
    }

    #[test]
    fn dashboard_assets_keep_all_market_render_contracts() {
        assert!(DASHBOARD_JS.contains("const NODE_META"));
        assert!(DASHBOARD_JS.contains("factorLabel"));
        assert!(DASHBOARD_JS.contains("renderSafely"));
        assert!(!DASHBOARD_JS.contains("NODE_LABELS"));
        for id in [
            "tickerTape",
            "overviewGauges",
            "marketLights",
            "overviewQuotes",
            "riskHeatmap",
            "proprietarySignals",
            "sourceHealth",
            "usMarket",
            "koreaMarket",
            "cryptoMarket",
        ] {
            assert!(
                DASHBOARD_HTML.contains(&format!("id=\"{id}\"")),
                "dashboard HTML is missing #{id}"
            );
        }
    }

    #[test]
    fn ticker_labels_expose_freshness_and_date() {
        let mut value = json!({
            "indicators": [{
                "key":"kospi",
                "label":"코스피",
                "freshness":"LATEST EOD",
                "observed_at":"2026-08-20"
            }]
        });
        decorate_ticker_freshness(&mut value);
        assert_eq!(
            value["indicators"][0]["label"],
            "코스피 · LATEST EOD · 2026-08-20"
        );
    }
}
