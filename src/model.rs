use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::base64::Base64;
use uuid::Uuid;

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct CaptureV2_2 {
    pub uuid: Uuid,
    pub host: String,
    pub cwd: String,
    pub cmd: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    #[serde_as(as = "Base64")]
    pub captured_output: Vec<u8>,
    pub message: String,
    pub exit_code: i32,
}
