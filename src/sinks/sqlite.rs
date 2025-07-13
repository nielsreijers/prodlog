use chrono::Duration;
use r2d2::PooledConnection;
use rusqlite::params;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::OptionalExtension;
use std::sync::Arc;
use uuid::Uuid;
use std::path::PathBuf;
use crate::{ helpers::compare_major_minor_versions, model::*, print_prodlog_message, print_prodlog_warning, prodlog_panic };
use super::Sink;
use r2d2_sqlite::SqliteConnectionManager;

pub struct SqliteSink {
    pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

fn migrate_up_one(
    conn: &PooledConnection<SqliteConnectionManager>,
    version: &str
) -> rusqlite::Result<String> {
    match version {
        "2.2.0" => {
            conn.execute(
                "ALTER TABLE prodlog_entries ADD COLUMN is_noop BOOLEAN DEFAULT FALSE",
                []
            )?;
            Ok("2.3.0".to_string())
        }
        "2.3.0" => {
            conn.execute("ALTER TABLE prodlog_entries ADD COLUMN local_user TEXT DEFAULT ''", [])?;
            conn.execute("ALTER TABLE prodlog_entries ADD COLUMN remote_user TEXT DEFAULT ''", [])?;
            conn.execute("ALTER TABLE prodlog_entries ADD COLUMN terminal_rows INT DEFAULT 0", [])?;
            conn.execute("ALTER TABLE prodlog_entries ADD COLUMN terminal_cols INT DEFAULT 0", [])?;
            Ok("2.4.0".to_string())
        }
        "2.4.0" => {
            // Add tasks table and task_id column to entries
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL
                )",
                []
            )?;
            conn.execute("ALTER TABLE prodlog_entries ADD COLUMN task_id INTEGER", [])?;
            Ok("2.5".to_string())
        }
        "2.5" => {
            // Add active task table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS active_task (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    task_id INTEGER,
                    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE SET NULL
                )",
                []
            )?;
            // Insert default row with no active task
            conn.execute("INSERT OR IGNORE INTO active_task (id, task_id) VALUES (1, NULL)", [])?;
            Ok("2.6".to_string())
        }
        _ => {
            prodlog_panic(
                &format!("Database schema version {} is not supported. Please upgrade Prodlog.", version)
            );
        }
    }
}

impl SqliteSink {
    fn get_schema_version(&self) -> rusqlite::Result<(Option<String>, bool)> {
        // TODO: Handle errors better instead of abusing InvalidParameterName
        let conn = self.pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let result = conn
            .query_row(
                "SELECT version, dirty FROM schema_migrations ORDER BY applied_at DESC LIMIT 1",
                [],
                |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, bool>(1)?))
            )
            .optional()?;
        Ok(result.unwrap_or((None, false)))
    }

    fn set_schema_version(&self, version: &str, is_dirty: bool) -> rusqlite::Result<()> {
        let conn = self.pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO schema_migrations (version, dirty, applied_at)
            VALUES (?1, ?2, STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'));",
            [version, if is_dirty { "1" } else { "0" }]
        )?;
        Ok(())
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT,
                dirty BOOLEAN,
                applied_at TIMESTAMP
            )",
            []
        )?;

        let current_version = env!("CARGO_PKG_VERSION").to_string();
        match self.get_schema_version()? {
            (Some(version), false) if compare_major_minor_versions(&version, &current_version) => {
                print_prodlog_message("Database is up to date.");
            }
            (Some(version), false) => {
                print_prodlog_warning(&format!("Upgrading database from version: {}", version));
                self.set_schema_version(version.as_str(), true)?;
                let new_version = migrate_up_one(&conn, &version)?;
                self.set_schema_version(new_version.as_str(), false)?;
                print_prodlog_warning(&format!("    ==> new version: {}", new_version));
                self.migrate()?;
            }
            (_, true) => {
                prodlog_panic("Database is dirty and needs to be fixed manually.");
            }
            (None, false) => {
                // Initialize the database schema
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
                        message TEXT,
                        is_noop BOOLEAN,
                        exit_code INTEGER,
                        local_user TEXT,
                        remote_user TEXT,
                        filename TEXT,
                        terminal_rows INTEGER,
                        terminal_cols INTEGER,
                        task_id INTEGER,
                        output BLOB,
                        original_content BLOB,
                        edited_content BLOB
                    );",
                    []
                )?;
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS tasks (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );",
                    []
                )?;
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS active_task (
                        id INTEGER PRIMARY KEY CHECK (id = 1),
                        task_id INTEGER,
                        FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE SET NULL
                    );",
                    []
                )?;
                conn.execute("INSERT OR IGNORE INTO active_task (id, task_id) VALUES (1, NULL)", [])?;
                self.set_schema_version(env!("CARGO_PKG_VERSION"), false)?;
            }
        }

        Ok(())
    }

    pub fn new(prodlog_file: &PathBuf) -> Self {
        let prodlog_file = prodlog_file.clone();
        let manager = SqliteConnectionManager::file(prodlog_file);
        let pool = match r2d2::Pool::new(manager) {
            Ok(pool) => pool,
            Err(e) => { prodlog_panic(&format!("Error creating sqlite pool: {}", e)) }
        };

        let sqlite_sink = SqliteSink { pool: Arc::new(pool) };
        let x = sqlite_sink.migrate();
        match x {
            Ok(_) => sqlite_sink,
            Err(e) => { prodlog_panic(&format!("Error migrating database: {}", e)) }
        }
    }
}


fn from_row_entry(row: &rusqlite::Row) -> rusqlite::Result<CaptureV2_4> {
    let capture_type: String = row.get("capture_type")?;
    let uuid_str: String = row.get("uuid")?;
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e|
        rusqlite::Error::InvalidParameterName(e.to_string())
    )?;
    Ok(CaptureV2_4 {
        capture_type: if capture_type == "run" {
            CaptureType::Run
        } else {
            CaptureType::Edit
        },
        uuid: uuid,
        host: row.get("host")?,
        cwd: row.get("cwd")?,
        cmd: row.get("cmd")?,
        start_time: row.get("start_time")?,
        duration_ms: row.get("duration_ms")?,
        message: row.get("message")?,
        is_noop: row.get("is_noop")?,
        exit_code: row.get("exit_code")?,
        local_user: row.get("local_user")?,
        remote_user: row.get("remote_user")?,
        filename: row.get("filename")?,
        terminal_rows: row.get("terminal_rows")?,
        terminal_cols: row.get("terminal_cols")?,
        task_id: row.get("task_id")?,
        captured_output: row.get("output")?,
        original_content: row.get("original_content")?,
        edited_content: row.get("edited_content")?,
    })
}

fn from_row_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        name: row.get("name")?,
        created_at: row.get("created_at")?,
    })
}


impl Sink for SqliteSink {
    fn add_entry(&self, capture: &CaptureV2_4) -> Result<(), std::io::Error> {
        let end_time = capture.start_time + Duration::milliseconds(capture.duration_ms as i64);
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        conn
            .execute(
                "INSERT OR REPLACE INTO prodlog_entries (capture_type, uuid, host, cwd, cmd, start_time, end_time, duration_ms, message, is_noop, exit_code, local_user, remote_user, filename, terminal_rows, terminal_cols, task_id, output, original_content, edited_content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
                params![
                    if capture.capture_type == CaptureType::Run {
                        "run"
                    } else {
                        "edit"
                    },
                    capture.uuid.to_string(),
                    &capture.host,
                    &capture.cwd,
                    &capture.cmd,
                    capture.start_time.to_rfc3339(),
                    end_time.to_rfc3339(),
                    capture.duration_ms as i64,
                    capture.message,
                    capture.is_noop,
                    capture.exit_code,
                    capture.local_user,
                    capture.remote_user,
                    capture.filename,
                    capture.terminal_rows,
                    capture.terminal_cols,
                    self.get_active_task()?,
                    &capture.captured_output,
                    capture.original_content,
                    capture.edited_content
                ]
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn get_entries(&self, filters: &super::Filters) -> Result<Vec<CaptureV2_4>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut query = String::from("SELECT * FROM prodlog_entries WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(date_from) = &filters.date_from {
            query.push_str(" AND start_time >= ?");
            params.push(Box::new(format!("{}T00:00:00", date_from)));
        }

        if let Some(date_to) = &filters.date_to {
            query.push_str(" AND start_time <= ?");
            params.push(Box::new(format!("{}T23:59:59", date_to)));
        }

        if let Some(host) = &filters.host {
            query.push_str(" AND host LIKE ?");
            params.push(Box::new(format!("%{}%", host)));
        }

        if let Some(command) = &filters.search {
            query.push_str(" AND (cmd LIKE ? OR message LIKE ?)");
            params.push(Box::new(format!("%{}%", command)));
            params.push(Box::new(format!("%{}%", command)));
        }

        if let Some(content) = &filters.search_content {
            query.push_str(" AND (cmd LIKE ? OR message LIKE ? OR CAST(output AS TEXT) LIKE ? OR CAST(original_content AS TEXT) LIKE ? OR CAST(edited_content AS TEXT) LIKE ?)");
            let search_pattern = format!("%{}%", content);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
        }

        if let Some(true) = &filters.show_noop {
            // Don't filter out no-op entries
        } else {
            query.push_str(" AND is_noop = 0");
        }

        query.push_str(" ORDER BY start_time DESC");

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let entries = stmt
            .query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
                from_row_entry(row)
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(entries)
    }

    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_4>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let uuid_str = uuid.to_string();
        match
            conn.query_row(
                "SELECT * FROM prodlog_entries WHERE uuid = ?",
                params![uuid_str],
                |row| { from_row_entry(row) }
            )
        {
            Ok(entry) => Ok(Some(entry)),
            Err(QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    fn create_task(&self, name: &str) -> Result<i64, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let created_at = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO tasks (name, created_at) VALUES (?1, ?2)",
            params![name, created_at]
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let task_id = conn.last_insert_rowid();
        Ok(task_id)
    }

    fn get_all_tasks(&self) -> Result<Vec<Task>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM tasks ORDER BY id DESC")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let tasks = stmt.query_map([], |row| { from_row_task(row) })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(tasks)
    }

    fn get_task_by_id(&self, id: i64) -> Result<Option<Task>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        match
            conn.query_row(
                "SELECT * FROM tasks WHERE id = ?",
                params![id],
                |row| { from_row_task(row) }
            )
        {
            Ok(entry) => Ok(Some(entry)),
            Err(QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    fn update_task_name(&self, task_id: i64, name: &str) -> Result<(), std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        conn.execute(
            "UPDATE tasks SET name = ? WHERE id = ?",
            params![name, task_id]
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(())
    }

    fn assign_entries_to_task(&self, entry_uuids: &[String], task_id: Option<i64>) -> Result<(), std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        for uuid in entry_uuids {
            conn.execute(
                "UPDATE prodlog_entries SET task_id = ? WHERE uuid = ?",
                params![task_id, uuid]
            ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        
        Ok(())
    }

    fn get_active_task(&self) -> Result<Option<i64>, std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        match conn.query_row(
            "SELECT task_id FROM active_task WHERE id = 1",
            [],
            |row| row.get::<_, Option<i64>>("task_id")
        ) {
            Ok(task_id) => Ok(task_id),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    fn set_active_task(&self, task_id: Option<i64>) -> Result<(), std::io::Error> {
        let conn = self.pool.get().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        conn.execute(
            "UPDATE active_task SET task_id = ? WHERE id = 1",
            params![task_id]
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(())
    }
}
