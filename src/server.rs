use crate::db::Db;
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
};

pub fn serve(host: &str, db_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(host)?;
    println!("ECONOMICS Radar: http://{host}");
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = [0u8; 2048];
        let n = stream.read(&mut buf)?;
        let req = String::from_utf8_lossy(&buf[..n]);
        let api = req.starts_with("GET /api/snapshot ");
        let db = Db::open(&db_path)?;
        let json = db.latest_snapshot_json()?.unwrap_or_else(|| "{}".into());
        let body = if api {
            json
        } else {
            format!(
                r#"<!doctype html><meta charset=utf-8><title>ECONOMICS Radar</title><style>body{{font-family:system-ui;max-width:900px;margin:40px auto;padding:0 20px;background:#111;color:#eee}}pre{{background:#1d1d1d;padding:20px;border-radius:12px;overflow:auto}}</style><h1>ECONOMICS Radar</h1><p>Rust low-memory financial risk monitor</p><pre id=x></pre><script>async function f(){{let r=await fetch('/api/snapshot');document.querySelector('#x').textContent=JSON.stringify(await r.json(),null,2)}}f();setInterval(f,60000)</script>"#
            )
        };
        let ct = if api {
            "application/json"
        } else {
            "text/html; charset=utf-8"
        };
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.as_bytes().len(),body)?;
    }
    Ok(())
}
