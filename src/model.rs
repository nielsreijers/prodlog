use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::base64::Base64;
use uuid::Uuid;

use crate::helpers;


#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CaptureType {
    Run,
    Edit,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct CaptureV2_3 {
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
    pub filename: String,
    #[serde_as(as = "Base64")]
    pub captured_output: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub original_content: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub edited_content: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub struct ProdlogDataV2_3 {
    pub prodlog_version: String,
    pub entries: Vec<CaptureV2_3>,
}
impl CaptureV2_3 {
    pub fn output_as_string(&self) -> String {
        String::from_utf8(self.captured_output.clone()).unwrap()
    }
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
#[derive(Serialize, Deserialize)]
pub struct ProdlogDataV2_2 {
    prodlog_version: String,
    entries: Vec<CaptureV2_2>,
}

#[derive(Serialize, Deserialize)]
pub struct CaptureV2_1 {
    uuid: Uuid,
    start_time: String,
    host: String,
    command: String,
    end_time: String,
    duration_ms: u64,
    log_filename: String,
    exit_code: i32,
    output: String,
}
#[derive(Serialize, Deserialize)]
pub struct ProdlogDataV2_1 {
    prodlog_version: String,
    entries: Vec<CaptureV2_1>,
}

#[derive(Serialize, Deserialize)]
pub struct CaptureV2_0 {
    start_time: String,
    host: String,
    command: String,
    end_time: String,
    duration_ms: u64,
    log_filename: String,
    prodlog_version: String,
    exit_code: i32,
    output: String,
}
#[derive(Serialize, Deserialize)]
pub struct ProdlogDataV2_0 {
    entries: Vec<CaptureV2_0>,
}

pub fn v2_0_to_v2_1(data: ProdlogDataV2_0) -> ProdlogDataV2_1 {
    ProdlogDataV2_1 {
        prodlog_version: "2.1.0".to_string(),
        entries: data.entries.into_iter().map(|e| CaptureV2_1 {
            uuid: Uuid::new_v4(),
            start_time: e.start_time,
            host: e.host,
            command: e.command,
            end_time: e.end_time,
            duration_ms: e.duration_ms,
            log_filename: e.log_filename,
            exit_code: e.exit_code,
            output: e.output,
        }).collect(),
    }
}

pub fn v2_1_to_v2_2(data: ProdlogDataV2_1) -> ProdlogDataV2_2 {
    ProdlogDataV2_2 {
        prodlog_version: "2.2.0".to_string(),
        entries: data.entries.into_iter().map(|e| CaptureV2_2 {
            capture_type: CaptureType::Run,
            uuid: Uuid::new_v4(),
            start_time: chrono::DateTime::parse_from_rfc3339(&e.start_time).unwrap().with_timezone(&chrono::Utc),
            host: e.host,
            cwd: "".to_string(),
            cmd: e.command,
            duration_ms: e.duration_ms,
            exit_code: e.exit_code,
            captured_output: helpers::base64_decode(&e.output),
            message: "".to_string(),
            filename: "".to_string(),
            original_content: "".as_bytes().to_vec(),
            edited_content: "".as_bytes().to_vec(),
        }).collect(),
    }
}

pub fn v2_2_to_v2_3(data: ProdlogDataV2_2) -> ProdlogDataV2_3 {
    ProdlogDataV2_3 {
        prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
        entries: data.entries.into_iter().map(|e| CaptureV2_3 {
            capture_type: e.capture_type,
            uuid: e.uuid,
            host: e.host.clone(),
            cwd: e.cwd.clone(),
            cmd: e.cmd.clone(),
            start_time: e.start_time,
            duration_ms: e.duration_ms,
            message: e.message.clone(),
            is_noop: false,
            exit_code: e.exit_code,
            captured_output: e.captured_output.clone(),
            filename: e.filename.clone(),
            original_content: e.original_content.clone(),
            edited_content: e.edited_content.clone(),
        }).collect(),
    }
}
