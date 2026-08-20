use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug)]
pub struct NewObservation {
    pub source: String,
    pub series: String,
    pub entity: String,
    pub observed_at: String,
    pub value: f64,
    pub released_at: Option<String>,
    pub source_asof: Option<String>,
    pub revision_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl NewObservation {
    pub fn simple(source: &str, series: &str, observed_at: &str, value: f64) -> Self {
        Self {
            source: source.into(),
            series: series.into(),
            entity: String::new(),
            observed_at: observed_at.into(),
            value,
            released_at: None,
            source_asof: None,
            revision_id: None,
            metadata: serde_json::Value::Null,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub observed_at: String,
    pub value: f64,
    pub released_at: Option<String>,
    pub source_asof: Option<String>,
    pub ingested_at: String,
}

#[derive(Clone, Debug, Default)]
pub struct SourceFreshness {
    pub latest_observed_at: Option<String>,
    pub latest_released_at: Option<String>,
    pub latest_ingested_at: Option<String>,
    pub revisions: usize,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-4096;
            PRAGMA mmap_size=0;
            PRAGMA temp_store=FILE;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS observations_v2(
              source TEXT NOT NULL,
              series TEXT NOT NULL,
              entity TEXT NOT NULL DEFAULT '',
              observed_at TEXT NOT NULL,
              value REAL NOT NULL,
              released_at TEXT,
              source_asof TEXT,
              revision_id TEXT NOT NULL,
              ingested_at TEXT NOT NULL,
              metadata TEXT NOT NULL DEFAULT 'null',
              PRIMARY KEY(source,series,entity,observed_at,revision_id)
            );
            CREATE INDEX IF NOT EXISTS idx_obs_v2_lookup
              ON observations_v2(source,series,observed_at DESC,released_at DESC,ingested_at DESC);
            CREATE INDEX IF NOT EXISTS idx_obs_v2_effective
              ON observations_v2(released_at,ingested_at);

            CREATE TABLE IF NOT EXISTS snapshots_v2(
              ts TEXT PRIMARY KEY,
              as_of TEXT NOT NULL,
              payload TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_v2_asof ON snapshots_v2(as_of DESC,ts DESC);
            "#,
        )?;

        let legacy_observations: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='observations')",
            [],
            |row| row.get(0),
        )?;
        if legacy_observations {
            conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO observations_v2(
                  source,series,entity,observed_at,value,released_at,source_asof,
                  revision_id,ingested_at,metadata
                )
                SELECT source,series,'',observed_at,value,released_at,NULL,
                       CASE WHEN COALESCE(vintage,'') <> '' THEN vintage
                            ELSE 'legacy:' || printf('%.17g',value) END,
                       ingested_at,'{"migrated_from":"observations"}'
                FROM observations;
                "#,
            )?;
        }

        let legacy_snapshots: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='snapshots')",
            [],
            |row| row.get(0),
        )?;
        if legacy_snapshots {
            conn.execute_batch(
                "INSERT OR IGNORE INTO snapshots_v2(ts,as_of,payload) SELECT ts,ts,payload FROM snapshots;",
            )?;
        }
        Ok(Self { conn })
    }

    pub fn put(&self, observation: &NewObservation) -> Result<bool> {
        if !observation.value.is_finite() {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "non-finite observation"),
            )));
        }
        let ingested_at = chrono::Utc::now().to_rfc3339();
        let revision_id = observation.revision_id.clone().unwrap_or_else(|| {
            observation
                .released_at
                .clone()
                .or_else(|| observation.source_asof.clone())
                .unwrap_or_else(|| format!("value:{:.17}", observation.value))
        });
        let metadata =
            serde_json::to_string(&observation.metadata).unwrap_or_else(|_| "null".into());
        let changed = self.conn.execute(
            "INSERT OR IGNORE INTO observations_v2(source,series,entity,observed_at,value,released_at,source_asof,revision_id,ingested_at,metadata) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                observation.source,
                observation.series,
                observation.entity,
                observation.observed_at,
                observation.value,
                observation.released_at,
                observation.source_asof,
                revision_id,
                ingested_at,
                metadata
            ],
        )?;
        Ok(changed == 1)
    }

    pub fn latest(&self, source: &str, series: &str, as_of: Option<&str>) -> Result<Option<Point>> {
        self.conn
            .query_row(
                r#"
                SELECT observed_at,value,released_at,source_asof,ingested_at
                FROM observations_v2
                WHERE source=?1 AND series=?2
                  AND (?3 IS NULL OR COALESCE(released_at,ingested_at) <= ?3)
                ORDER BY observed_at DESC,COALESCE(released_at,ingested_at) DESC,revision_id DESC
                LIMIT 1
                "#,
                params![source, series, as_of],
                |row| {
                    Ok(Point {
                        observed_at: row.get(0)?,
                        value: row.get(1)?,
                        released_at: row.get(2)?,
                        source_asof: row.get(3)?,
                        ingested_at: row.get(4)?,
                    })
                },
            )
            .optional()
    }

    pub fn recent(
        &self,
        source: &str,
        series: &str,
        limit: usize,
        as_of: Option<&str>,
    ) -> Result<Vec<Point>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT observed_at,value,released_at,source_asof,ingested_at FROM (
              SELECT observed_at,value,released_at,source_asof,ingested_at,
                     ROW_NUMBER() OVER (
                       PARTITION BY entity,observed_at
                       ORDER BY COALESCE(released_at,ingested_at) DESC,revision_id DESC
                     ) AS revision_rank
              FROM observations_v2
              WHERE source=?1 AND series=?2
                AND (?3 IS NULL OR COALESCE(released_at,ingested_at) <= ?3)
            )
            WHERE revision_rank=1
            ORDER BY observed_at DESC
            LIMIT ?4
            "#,
        )?;
        let rows = statement.query_map(params![source, series, as_of, limit as i64], |row| {
            Ok(Point {
                observed_at: row.get(0)?,
                value: row.get(1)?,
                released_at: row.get(2)?,
                source_asof: row.get(3)?,
                ingested_at: row.get(4)?,
            })
        })?;
        let mut points = rows.collect::<Result<Vec<_>>>()?;
        points.reverse();
        Ok(points)
    }

    pub fn source_freshness(&self, source: &str, as_of: Option<&str>) -> Result<SourceFreshness> {
        self.conn.query_row(
            r#"
            SELECT MAX(observed_at),MAX(max_released_at),MAX(max_ingested_at),
                   COALESCE(SUM(CASE WHEN revision_count > 1 THEN revision_count - 1 ELSE 0 END),0)
            FROM (
                SELECT series,entity,observed_at,
                       MAX(released_at) AS max_released_at,
                       MAX(ingested_at) AS max_ingested_at,
                       COUNT(*) AS revision_count
                FROM observations_v2
                WHERE source=?1 AND (?2 IS NULL OR COALESCE(released_at,ingested_at) <= ?2)
                GROUP BY series,entity,observed_at
            )
            "#,
            params![source, as_of],
            |row| {
                Ok(SourceFreshness {
                    latest_observed_at: row.get(0)?,
                    latest_released_at: row.get(1)?,
                    latest_ingested_at: row.get(2)?,
                    revisions: row.get::<_, i64>(3)? as usize,
                })
            },
        )
    }

    pub fn observation_dates(&self, start: &str, end: &str) -> Result<Vec<String>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT DISTINCT substr(observed_at,1,10) AS day
            FROM observations_v2
            WHERE substr(observed_at,1,10) BETWEEN ?1 AND ?2
            ORDER BY day
            "#,
        )?;
        let rows = statement.query_map(params![start, end], |row| row.get(0))?;
        rows.collect()
    }

    pub fn save_snapshot(&self, snapshot: &crate::engine::Snapshot) -> Result<()> {
        let payload = serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".into());
        self.conn.execute(
            "INSERT OR REPLACE INTO snapshots_v2(ts,as_of,payload) VALUES(?1,?2,?3)",
            params![snapshot.ts, snapshot.as_of, payload],
        )?;
        Ok(())
    }

    pub fn latest_snapshot_json(&self) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT payload FROM snapshots_v2 ORDER BY as_of DESC,ts DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn latest_snapshot_before(&self, as_of: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT payload FROM snapshots_v2 WHERE as_of < ?1 ORDER BY as_of DESC,ts DESC LIMIT 1",
                [as_of],
                |row| row.get(0),
            )
            .optional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_are_preserved_and_selected_as_of() {
        let temp = tempfile::tempdir().unwrap();
        let db = Db::open(&temp.path().join("test.db")).unwrap();
        let mut first = NewObservation::simple("fred", "X", "2020-01-01", 1.0);
        first.released_at = Some("2020-02-01T00:00:00Z".into());
        first.revision_id = Some("v1".into());
        assert!(db.put(&first).unwrap());
        let mut revised = first.clone();
        revised.value = 2.0;
        revised.released_at = Some("2020-03-01T00:00:00Z".into());
        revised.revision_id = Some("v2".into());
        assert!(db.put(&revised).unwrap());

        assert_eq!(
            db.latest("fred", "X", Some("2020-02-15T00:00:00Z"))
                .unwrap()
                .unwrap()
                .value,
            1.0
        );
        assert_eq!(db.latest("fred", "X", None).unwrap().unwrap().value, 2.0);
        assert_eq!(db.recent("fred", "X", 10, None).unwrap().len(), 1);
    }
}
