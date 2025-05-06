use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::base64::Base64;
use uuid::Uuid;


#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CaptureType {
    Run,
    Edit,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct CaptureV2_2 {
    pub capture_type: CaptureType,
    pub uuid: Uuid,
    pub host: String,
    pub cwd: String,
    pub cmd: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub message: String,
    pub exit_code: i32,
    #[serde_as(as = "Base64")]
    pub captured_output: Vec<u8>,
    pub filename: String,
    #[serde_as(as = "Base64")]
    pub original_content: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub edited_content: Vec<u8>,
}

impl CaptureV2_2 {
    pub fn output_as_string(&self) -> String {
        String::from_utf8(self.captured_output.clone()).unwrap()
    }
}
