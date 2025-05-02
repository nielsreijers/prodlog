use core::panic;
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::CaptureState;
use super::{get_short_command, Sink};
use base64;
use base64::Engine as _;
use uuid::Uuid;

pub struct JsonSink {
    prodlog_file: PathBuf,
}

impl JsonSink {
    pub fn new(prodlog_dir: PathBuf) -> Self {
        std::fs::create_dir_all(prodlog_dir.clone()).unwrap();
        let prodlog_file = prodlog_dir.join("prodlog.json");
        // Check the file is valid so we don't crash while logging a command if it's not
        read_prodlog_data(&prodlog_file).unwrap();
        return Self { prodlog_file };
    }
}

#[derive(Serialize, Deserialize)]
struct ProdlogEntryV2_0 {
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
struct ProdlogDataV2_0 {
    entries: Vec<ProdlogEntryV2_0>,
}


#[derive(Serialize, Deserialize)]
struct ProdlogEntryV2_1 {
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
struct ProdlogDataV2_1 {
    prodlog_version: String,
    entries: Vec<ProdlogEntryV2_1>,
}

#[derive(Serialize, Deserialize)]
struct ProdlogEntryV2_2 {
    uuid: Uuid,
    start_time: String,
    host: String,
    command: String,
    end_time: String,
    duration_ms: u64,
    log_filename: String,
    exit_code: i32,
    output: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct ProdlogDataV2_2 {
    prodlog_version: String,
    entries: Vec<ProdlogEntryV2_2>,
}

// TODO: this is just temporary, we should log the output as base64 to the json file directly
fn get_output_log_filename (capture: &CaptureState) -> String {
    let formatted_time = capture.start_time.format("%Y%m%d_%H%M%S").to_string();
    let short_cmd = get_short_command(&capture.cmd).replace(" ", "_");
    format!("prodlog_output/{}/{}-{}.md", capture.host, formatted_time, short_cmd)
}

fn v2_0_to_v2_1(data: ProdlogDataV2_0) -> ProdlogDataV2_1 {
    ProdlogDataV2_1 {
        prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
        entries: data.entries.into_iter().map(|e| ProdlogEntryV2_1 {
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


fn v2_1_to_v2_2(data: ProdlogDataV2_1) -> ProdlogDataV2_2 {
    ProdlogDataV2_2 {
        prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
        entries: data.entries.into_iter().map(|e| ProdlogEntryV2_2 {
            uuid: Uuid::new_v4(),
            start_time: e.start_time,
            host: e.host,
            command: e.command,
            end_time: e.end_time,
            duration_ms: e.duration_ms,
            log_filename: e.log_filename,
            exit_code: e.exit_code,
            output: e.output,
            message: "".to_string()
        }).collect(),
    }
}

fn read_prodlog_data(json_path: &PathBuf) -> Result<ProdlogDataV2_2, std::io::Error> {
    // Read existing JSON file
    if let Ok(content) = fs::read_to_string(&json_path) {
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(data);
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_1_to_v2_2(data));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_1_to_v2_2(v2_0_to_v2_1(data)));
        }
        panic!("Failed to read prodlog data from {}", json_path.display());
    } else {
        Ok(ProdlogDataV2_2 {
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
            entries: vec![],
        })
    }
}

impl Sink for JsonSink {
    fn add_entry(&mut self, capture: &CaptureState, exit_code: i32, end_time: DateTime<Utc>) -> Result<(), std::io::Error> {
        // Read existing JSON file
        let mut prodlog_data = read_prodlog_data(&self.prodlog_file)?;

        // Add new entry
        let host = &capture.host;
        let cmd_long = &capture.cmd;
        let log_filename = get_output_log_filename(&capture);
        let duration_ms = end_time.signed_duration_since(capture.start_time).num_milliseconds() as u64;
        prodlog_data.entries.push(ProdlogEntryV2_2 {
            uuid: capture.uuid,
            host: host.to_string(),
            start_time: capture.start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            duration_ms,
            command: cmd_long.to_string(),
            log_filename: log_filename.to_string(),
            exit_code,
            output: base64::engine::general_purpose::STANDARD.encode(&capture.captured_output),
            message: capture.message.to_string(),
        });

        // Write updated JSON file
        fs::write(&self.prodlog_file, serde_json::to_string_pretty(&prodlog_data)?)?;

        Ok(())
    }
}
