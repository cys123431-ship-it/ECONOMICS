use crate::{dashboard, db::Db, refresh::RefreshControl};
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
            let snapshot = db
                .latest_snapshot_json()?
                .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok())
                .unwrap_or_else(|| serde_json::json!({"status":"no_snapshot"}));
            let body = serde_json::to_string(&serde_json::json!({
                "snapshot": snapshot,
                "dashboard": dashboard::build(&db)?,
            }))?;
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
        assert!(DASHBOARD_JS.contains("대시보드 데이터를 불러오지 못했습니다"));
        assert!(DASHBOARD_JS.contains("renderMarket"));
        assert!(DASHBOARD_JS.contains("/api/refresh-status"));
        assert!(DASHBOARD_JS.contains("method: 'POST'"));
        assert!(DASHBOARD_HTML.contains("data-tab=\"us\""));
        assert!(DASHBOARD_HTML.contains("data-tab=\"korea\""));
        assert!(DASHBOARD_HTML.contains("data-tab=\"crypto\""));
        assert!(DASHBOARD_CSS.contains(".dial"));
    }
}
