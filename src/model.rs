use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use serde_with::serde_as;
use serde_with::base64::Base64;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CaptureType {
    Run,
    Edit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct CaptureV2_4 {
    pub capture_type: CaptureType,
    pub uuid: Uuid,
    pub host: String,
    pub cwd: String,
    pub cmd: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub message: String,
    pub is_noop: bool,
    pub exit_code: i32,
    pub local_user: String,
    pub remote_user: String,
    pub filename: String,
    pub terminal_rows: u16,
    pub terminal_cols: u16,
    pub task_id: Option<i64>,
    #[serde_as(as = "Base64")]
    pub captured_output: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub original_content: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub edited_content: Vec<u8>,
}

// Lightweight version for index page - excludes large content fields
#[derive(Serialize, Deserialize, Clone)]
pub struct CaptureV2_4Summary {
    pub capture_type: CaptureType,
    pub uuid: Uuid,
    pub host: String,
    pub cwd: String,
    pub cmd: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub message: String,
    pub is_noop: bool,
    pub exit_code: i32,
    pub local_user: String,
    pub remote_user: String,
    pub filename: String,
    pub terminal_rows: u16,
    pub terminal_cols: u16,
    pub task_id: Option<i64>,
}

impl From<&CaptureV2_4> for CaptureV2_4Summary {
    fn from(entry: &CaptureV2_4) -> Self {
        CaptureV2_4Summary {
            capture_type: entry.capture_type.clone(),
            uuid: entry.uuid,
            host: entry.host.clone(),
            cwd: entry.cwd.clone(),
            cmd: entry.cmd.clone(),
            start_time: entry.start_time,
            duration_ms: entry.duration_ms,
            message: entry.message.clone(),
            is_noop: entry.is_noop,
            exit_code: entry.exit_code,
            local_user: entry.local_user.clone(),
            remote_user: entry.remote_user.clone(),
            filename: entry.filename.clone(),
            terminal_rows: entry.terminal_rows,
            terminal_cols: entry.terminal_cols,
            task_id: entry.task_id,
        }
    }
}
