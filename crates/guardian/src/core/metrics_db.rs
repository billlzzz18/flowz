use crate::detectors::event_behavior_detector::{EventBehaviorDetector, MetricEventMetadata};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

pub const SCHEMA_VERSION: i64 = 5;
#[derive(Debug, Clone, PartialEq)]
pub struct BackfillSummary {
    pub scanned: usize,
    pub updated: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CrossSessionLink {
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub trace_id: Option<String>,
    pub event_count: usize,
    pub first_ts: u64,
    pub last_ts: u64,
}

pub struct MetricsDb {
    conn: Connection,
}
impl MetricsDb {
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let db = Self {
            conn: Connection::open_in_memory()?,
        };
        db.migrate()?;
        Ok(db)
    }
    pub fn open(path: &str) -> rusqlite::Result<Self> {
        let db = Self { conn: Connection::open(path)? };
        db.migrate()?;
        Ok(db)
    }
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS schema_version(version INTEGER NOT NULL); INSERT INTO schema_version(version) SELECT 0 WHERE NOT EXISTS(SELECT 1 FROM schema_version);")?;
        let current: i64 = self
            .conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))?;
        if current < 4 {
            self.conn.execute_batch("CREATE TABLE IF NOT EXISTS metrics(id INTEGER PRIMARY KEY AUTOINCREMENT,event_json TEXT NOT NULL,delivered_ts INTEGER,event_ts INTEGER,event_kind INTEGER,trace_id TEXT,session_id TEXT,parent_session_id TEXT,tool TEXT,external_session_id TEXT,external_parent_session_id TEXT,external_event_id TEXT,external_parent_event_id TEXT,external_tool_use_id TEXT); CREATE INDEX IF NOT EXISTS metrics_event_ts_kind ON metrics(event_ts,event_kind,id) WHERE event_ts IS NOT NULL AND event_kind IS NOT NULL; CREATE INDEX IF NOT EXISTS metrics_session_kind_ts ON metrics(session_id,event_kind,event_ts,id) WHERE session_id IS NOT NULL AND event_kind IS NOT NULL AND event_ts IS NOT NULL; CREATE INDEX IF NOT EXISTS metrics_parent_session_kind_ts ON metrics(parent_session_id,event_kind,event_ts,id) WHERE parent_session_id IS NOT NULL AND event_kind IS NOT NULL AND event_ts IS NOT NULL; UPDATE schema_version SET version=4;")?;
        }
        if current < 5 {
            self.conn.execute_batch(
                r"CREATE TABLE IF NOT EXISTS commands(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    ts TEXT NOT NULL,
                    cwd TEXT NOT NULL,
                    cmd TEXT NOT NULL,
                    exit_code INTEGER,
                    session_id TEXT
                );
                CREATE INDEX IF NOT EXISTS commands_ts ON commands(ts);
                CREATE INDEX IF NOT EXISTS commands_session ON commands(session_id);
                UPDATE schema_version SET version=5;",
            )?;
        }
        Ok(())
    }
    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
    }
    pub fn insert_event(&self, json: &str, delivered_ts: Option<u64>) -> rusqlite::Result<i64> {
        let m = EventBehaviorDetector::extract_metadata(json);
        self.conn.execute("INSERT INTO metrics(event_json,delivered_ts,event_ts,event_kind,trace_id,session_id,parent_session_id,tool,external_session_id,external_parent_session_id,external_event_id,external_parent_event_id,external_tool_use_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",params![json,delivered_ts.map(|x|x as i64),m.as_ref().map(|x|x.event_ts as i64),m.as_ref().map(|x|x.event_kind as i64),m.as_ref().and_then(|x|x.trace_id.as_deref()),m.as_ref().and_then(|x|x.session_id.as_deref()),m.as_ref().and_then(|x|x.parent_session_id.as_deref()),m.as_ref().and_then(|x|x.tool.as_deref()),m.as_ref().and_then(|x|x.external_session_id.as_deref()),m.as_ref().and_then(|x|x.external_parent_session_id.as_deref()),m.as_ref().and_then(|x|x.external_event_id.as_deref()),m.as_ref().and_then(|x|x.external_parent_event_id.as_deref()),m.as_ref().and_then(|x|x.external_tool_use_id.as_deref())])?;
        Ok(self.conn.last_insert_rowid())
    }
    pub fn insert_events<I, S>(&self, events: I) -> rusqlite::Result<usize>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut n = 0;
        for e in events {
            self.insert_event(e.as_ref(), None)?;
            n += 1
        }
        Ok(n)
    }
    pub fn backfill_event_metadata_batch(&self, limit: usize) -> rusqlite::Result<BackfillSummary> {
        Ok(self.backfill_event_metadata_batch_after(limit, 0)?.0)
    }
    fn backfill_event_metadata_batch_after(
        &self,
        limit: usize,
        after_id: i64,
    ) -> rusqlite::Result<(BackfillSummary, i64)> {
        let mut stmt=self.conn.prepare("SELECT id,event_json FROM metrics WHERE id>?1 AND (event_ts IS NULL OR event_kind IS NULL) ORDER BY id LIMIT ?2")?;
        let rows = stmt
            .query_map(params![after_id, limit as i64], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        let mut updated = 0;
        for (id, json) in &rows {
            if let Some(m) = EventBehaviorDetector::extract_metadata(json) {
                self.update_metadata(*id, &m)?;
                updated += 1
            }
        }
        let last_id = rows.last().map(|(id, _)| *id).unwrap_or(after_id);
        Ok((BackfillSummary { scanned: rows.len(), updated }, last_id))
    }
    fn update_metadata(&self, id: i64, m: &MetricEventMetadata) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE metrics SET event_ts=?1,event_kind=?2,trace_id=?3,session_id=?4,parent_session_id=?5,tool=?6,external_session_id=?7,external_parent_session_id=?8,external_event_id=?9,external_parent_event_id=?10,external_tool_use_id=?11 WHERE id=?12",params![m.event_ts as i64,m.event_kind as i64,m.trace_id,m.session_id,m.parent_session_id,m.tool,m.external_session_id,m.external_parent_session_id,m.external_event_id,m.external_parent_event_id,m.external_tool_use_id,id])?;
        Ok(())
    }
    pub fn backfill_event_metadata(&self, batch: usize) -> rusqlite::Result<BackfillSummary> {
        let mut total = BackfillSummary { scanned: 0, updated: 0 };
        let mut cursor = 0;
        loop {
            let (s, last_id) = self.backfill_event_metadata_batch_after(batch.max(1), cursor)?;
            total.scanned += s.scanned;
            total.updated += s.updated;
            cursor = last_id;
            if s.scanned < batch.max(1) {
                break;
            }
        }
        Ok(total)
    }
    pub fn count(&self) -> rusqlite::Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM metrics", [], |r| r.get::<_, i64>(0))
            .map(|x| x as usize)
    }
    pub fn cross_session_links(&self) -> rusqlite::Result<Vec<CrossSessionLink>> {
        let mut st=self.conn.prepare("SELECT session_id,parent_session_id,MIN(trace_id),COUNT(*),MIN(event_ts),MAX(event_ts) FROM metrics WHERE session_id IS NOT NULL AND event_ts IS NOT NULL GROUP BY session_id,parent_session_id ORDER BY MIN(event_ts)")?;
        let rows = st
            .query_map([], |r| {
                Ok(CrossSessionLink {
                    session_id: r.get(0)?,
                    parent_session_id: r.get(1)?,
                    trace_id: r.get(2)?,
                    event_count: r.get::<_, i64>(3)? as usize,
                    first_ts: r.get::<_, i64>(4)? as u64,
                    last_ts: r.get::<_, i64>(5)? as u64,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn session_event_count(&self, session: &str) -> rusqlite::Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM metrics WHERE session_id=?1", params![session], |r| {
                r.get::<_, i64>(0)
            })
            .map(|x| x as usize)
    }
    pub fn optional_event_ts(&self, id: i64) -> rusqlite::Result<Option<u64>> {
        self.conn
            .query_row("SELECT event_ts FROM metrics WHERE id=?1", [id], |r| {
                r.get::<_, Option<i64>>(0)
            })
            .map(|x| x.map(|v| v as u64))
            .optional()
            .map(|x| x.flatten())
    }

    /// Insert a shell command log entry (TSV: ts, cwd, cmd).
    pub fn insert_command(
        &self,
        ts: &str,
        cwd: &str,
        cmd: &str,
        session_id: Option<&str>,
    ) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO commands(ts, cwd, cmd, session_id) VALUES(?1,?2,?3,?4)",
            params![ts, cwd, cmd, session_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Bulk import a TSV command log file.
    pub fn import_command_log(
        &self,
        content: &str,
        session_id: Option<&str>,
    ) -> rusqlite::Result<usize> {
        let mut n = 0;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() < 3 {
                continue;
            }
            self.insert_command(parts[0], parts[1], parts[2], session_id)?;
            n += 1;
        }
        Ok(n)
    }

    /// Query recent commands (for detector input).
    pub fn recent_commands(&self, limit: usize) -> rusqlite::Result<Vec<CommandEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT ts, cwd, cmd, exit_code, session_id \\
             FROM commands ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit as i64], |r| {
                Ok(CommandEntry {
                    ts: r.get(0)?,
                    cwd: r.get(1)?,
                    cmd: r.get(2)?,
                    exit_code: r.get(3)?,
                    session_id: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

/// Returns the default database path: $HOME/.guardian/metrics.db (or USERPROFILE on Windows)
pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".guardian").join("metrics.db")
}

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub ts: String,
    pub cwd: String,
    pub cmd: String,
    pub exit_code: Option<i32>,
    pub session_id: Option<String>,
}
