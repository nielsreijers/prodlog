use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use crate::CaptureState;
use super::Sink;

pub struct SqliteSink {
    conn: Connection,
}

impl SqliteSink {
    pub fn new(prodlog_dir: PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(prodlog_dir.clone()).unwrap();
        let prodlog_db_file = prodlog_dir.join("prodlog.sqlite");
        
        let conn = Connection::open(prodlog_db_file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prodlog_entries (
                uuid TEXT PRIMARY KEY,
                host TEXT,
                cwd TEXT,
                cmd TEXT,
                start_time TEXT,
                end_time TEXT,
                duration_ms INTEGER,
                exit_code INTEGER,
                output BLOB
            )",
            [],
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(SqliteSink { conn })
    }
}

impl Sink for SqliteSink {
    fn add_entry(&mut self, capture: &CaptureState, exit_code: i32, end_time: DateTime<Utc>) -> Result<(), std::io::Error> {
        let duration_ms = end_time.signed_duration_since(capture.start_time).num_milliseconds() as u64;
        let output = &capture.captured_output;
        self.conn.execute(
            "INSERT INTO prodlog_entries (uuid, host, cwd, cmd, start_time, end_time, duration_ms, exit_code, output)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                capture.uuid.to_string(),
                &capture.host,
                &capture.cwd,
                &capture.cmd,
                capture.start_time.to_rfc3339(),
                end_time.to_rfc3339(),
                duration_ms as i64,
                exit_code,
                output,
            ],
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
} 