use crate::db::Db;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

pub fn serve(host: &str, db_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(host)?;
    println!("ECONOMICS Radar: http://{host}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle(stream, &db_path) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle(mut stream: TcpStream, db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0u8; 8192];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

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
        "/" => respond(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            r#"<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ECONOMICS Radar</title><style>body{font-family:system-ui;max-width:1000px;margin:40px auto;padding:0 20px;background:#111;color:#eee}pre{background:#1d1d1d;padding:20px;border-radius:12px;overflow:auto}.muted{color:#aaa}</style><h1>ECONOMICS Radar</h1><p class="muted">Canonical v4 ULTRA rule engine · missing data is shown as null</p><pre id="snapshot">loading…</pre><script>async function refresh(){const response=await fetch('/api/snapshot',{cache:'no-store'});document.querySelector('#snapshot').textContent=JSON.stringify(await response.json(),null,2)}refresh();setInterval(refresh,60000)</script></html>"#,
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
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; style-src 'unsafe-inline'\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(())
}
