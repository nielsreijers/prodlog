use chrono::Duration;
use rusqlite::params;
use rusqlite::Error::QueryReturnedNoRows;
use std::sync::Arc;
use uuid::Uuid;
use std::path::PathBuf;
use crate::model::{CaptureType, CaptureV2_2};
use super::{Sink, UiSource};
use r2d2_sqlite::SqliteConnectionManager;

pub struct SqliteSink {
    pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

impl SqliteSink {
    pub fn new(prodlog_file: &PathBuf) -> Result<Self, std::io::Error> {
        let prodlog_file = prodlog_file.clone();
        let manager = SqliteConnectionManager::file(prodlog_file);
        let pool = r2d2::Pool::new(manager).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Initialize the database schema
        let conn = pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
        Ok(SqliteSink { pool: Arc::new(pool) })
    }
}

impl Sink for SqliteSink {
    fn add_entry(&mut self, capture: &CaptureV2_2) -> Result<(), std::io::Error> {
        let end_time = capture.start_time + Duration::milliseconds(capture.duration_ms as i64);
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        conn.execute(
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

fn from_row(row: &rusqlite::Row) -> rusqlite::Result<CaptureV2_2> {
    let capture_type: String = row.get("capture_type")?;
    let uuid_str: String = row.get("uuid")?;
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    Ok(CaptureV2_2{
        capture_type: if capture_type == "run" { CaptureType::Run } else { CaptureType::Edit },
        uuid: uuid,
        host: row.get("host")?,
        cwd: row.get("cwd")?,
        cmd: row.get("cmd")?,
        start_time: row.get("start_time")?,
        duration_ms: row.get("duration_ms")?,
        message: row.get("message")?,
        exit_code: row.get("exit_code")?,
        captured_output: row.get("output")?,
        filename: row.get("filename")?,
        original_content: row.get("original_content")?,
        edited_content: row.get("edited_content")?,
    })
}

impl UiSource for SqliteSink {
    fn get_entries(&self, filters: &super::Filters) -> Result<Vec<CaptureV2_2>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let mut query = String::from("SELECT * FROM prodlog_entries WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(date) = &filters.date {
            query.push_str(" AND start_time LIKE ?");
            params.push(Box::new(format!("{}%", date)));
        }
        
        if let Some(host) = &filters.host {
            query.push_str(" AND host LIKE ?");
            params.push(Box::new(format!("%{}%", host)));
        }
        
        if let Some(command) = &filters.command {
            query.push_str(" AND cmd LIKE ?");
            params.push(Box::new(format!("%{}%", command)));
        }
        
        if let Some(output) = &filters.output {
            query.push_str(" AND output LIKE ?");
            params.push(Box::new(format!("%{}%", output)));
        }

        query.push_str(" ORDER BY start_time DESC");
        
        let mut stmt = conn.prepare(&query).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let entries = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
            from_row(row)
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // If there's an output filter, we need to filter in Rust since we can't easily search in BLOB
        if let Some(output_filter) = &filters.output {
            if !output_filter.is_empty() {
                return Ok(entries.into_iter()
                    .filter(|entry| {
                        let output = entry.output_as_string();
                        output.to_lowercase().contains(&output_filter.to_lowercase())
                    })
                    .collect());
            }
        }
        
        Ok(entries)
    }

    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_2>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let uuid_str = uuid.to_string();
        match conn.query_row("SELECT * FROM prodlog_entries WHERE uuid = ?", params![uuid_str], |row| {
            from_row(row)
        }) {
            Ok(entry) => Ok(Some(entry)),
            Err(QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }   
    }
}                