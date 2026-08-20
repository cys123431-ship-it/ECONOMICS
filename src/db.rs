use rusqlite::{params, Connection, OptionalExtension, Result};
use std::{fs, path::Path};

pub struct Db { conn: Connection }

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?; }
        let conn = Connection::open(path)?;
        conn.execute_batch(r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-4096;
            PRAGMA mmap_size=0;
            PRAGMA temp_store=FILE;
            CREATE TABLE IF NOT EXISTS observations(
              source TEXT NOT NULL,
              series TEXT NOT NULL,
              observed_at TEXT NOT NULL,
              value REAL NOT NULL,
              released_at TEXT,
              vintage TEXT NOT NULL DEFAULT '',
              ingested_at TEXT NOT NULL,
              PRIMARY KEY(source,series,observed_at,vintage)
            );
            CREATE INDEX IF NOT EXISTS idx_obs_lookup ON observations(source,series,observed_at DESC);
            CREATE TABLE IF NOT EXISTS snapshots(
              ts TEXT PRIMARY KEY,
              global_risk REAL NOT NULL,
              stress REAL NOT NULL,
              vulnerability REAL NOT NULL,
              resilience REAL NOT NULL,
              confidence REAL NOT NULL,
              diffusion INTEGER NOT NULL,
              stage INTEGER NOT NULL,
              payload TEXT NOT NULL
            );
        "#)?;
        Ok(Self { conn })
    }

    pub fn put(&self, source:&str, series:&str, observed_at:&str, value:f64, vintage:Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO observations(source,series,observed_at,value,vintage,ingested_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![source,series,observed_at,value,vintage.unwrap_or(""),now])?;
        Ok(())
    }

    pub fn latest(&self, source:&str, series:&str) -> Result<Option<f64>> {
        self.conn.query_row(
            "SELECT value FROM observations WHERE source=?1 AND series=?2 ORDER BY observed_at DESC LIMIT 1",
            params![source,series], |r| r.get(0)).optional()
    }

    pub fn recent(&self, source:&str, series:&str, limit:usize) -> Result<Vec<f64>> {
        let mut st = self.conn.prepare("SELECT value FROM observations WHERE source=?1 AND series=?2 ORDER BY observed_at DESC LIMIT ?3")?;
        let rows=st.query_map(params![source,series,limit as i64], |r| r.get::<_,f64>(0))?;
        let mut v=Vec::with_capacity(limit.min(256));
        for x in rows { v.push(x?); }
        v.reverse(); Ok(v)
    }

    pub fn save_snapshot(&self, s:&crate::engine::Snapshot) -> Result<()> {
        let payload=serde_json::to_string(s).unwrap_or_else(|_| "{}".into());
        self.conn.execute("INSERT OR REPLACE INTO snapshots(ts,global_risk,stress,vulnerability,resilience,confidence,diffusion,stage,payload) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![s.ts,s.global_risk,s.stress,s.vulnerability,s.resilience,s.confidence,s.diffusion,s.stage,payload])?;
        Ok(())
    }

    pub fn latest_snapshot_json(&self) -> Result<Option<String>> {
        self.conn.query_row("SELECT payload FROM snapshots ORDER BY ts DESC LIMIT 1", [], |r| r.get(0)).optional()
    }
}
