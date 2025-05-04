use chrono::Duration;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use crate::model::{CaptureType, CaptureV2_2};
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
                capture_type TEXT,
                uuid TEXT PRIMARY KEY,
                host TEXT,
                cwd TEXT,
                cmd TEXT,
                start_time TEXT,
                end_time TEXT,
                duration_ms INTEGER,
                exit_code INTEGER,
                output BLOB,
                message TEXT,
                filename TEXT,
                original_content BLOB,
                edited_content BLOB
            )",
            [],
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(SqliteSink { conn })
    }
}

impl Sink for SqliteSink {
    fn add_entry(&mut self, capture: &CaptureV2_2) -> Result<(), std::io::Error> {
        let end_time = capture.start_time + Duration::milliseconds(capture.duration_ms as i64);
        self.conn.execute(
            "INSERT INTO prodlog_entries (capture_type, uuid, host, cwd, cmd, start_time, end_time, duration_ms, exit_code, output, message, filename, original_content, edited_content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                if capture.capture_type == CaptureType::Run { "run" } else { "edit" },
                capture.uuid.to_string(),
                &capture.host,
                &capture.cwd,
                &capture.cmd,
                capture.start_time.to_rfc3339(),
                end_time.to_rfc3339(),
                capture.duration_ms as i64,
                capture.exit_code,
                &capture.captured_output,
                capture.message,
                capture.filename,
                capture.original_content,
                capture.edited_content,
            ],
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
} 