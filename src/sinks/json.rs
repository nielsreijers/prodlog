use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::CaptureState;
use super::{get_short_command, Sink};

pub struct JsonSink {
    prodlog_dir: PathBuf,
}

impl JsonSink {
    pub fn new(prodlog_dir: PathBuf) -> Self {
        Self { prodlog_dir }
    }
}

#[derive(Serialize, Deserialize)]
struct ProdlogEntry {
    start_time: String,
    host: String,
    command: String,
    end_time: String,
    duration_ms: u64,
    log_filename: String,
    prodlog_version: String,
    exit_code: i32,
}

#[derive(Serialize, Deserialize)]
struct ProdlogData {
    entries: Vec<ProdlogEntry>,
}

// TODO: this is just temporary, we should log the output as base64 to the json file directly
fn get_output_log_filename (capture: &CaptureState) -> String {
    let formatted_time = capture.start_time.format("%Y%m%d_%H%M%S").to_string();
    let short_cmd = get_short_command(&capture.cmd).replace(" ", "_");
    format!("prodlog_output/{}/{}-{}.md", capture.host, formatted_time, short_cmd)
}


impl Sink for JsonSink {
    fn add_entry(&mut self, capture: &CaptureState, exit_code: i32, end_time: DateTime<Utc>) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(self.prodlog_dir.clone()).unwrap();

        // Read existing JSON file
        let json_path = self.prodlog_dir.join("prodlog.json");
        let mut prodlog_data = if let Ok(content) = fs::read_to_string(&json_path) {
            serde_json::from_str(&content).unwrap_or(ProdlogData { entries: Vec::new() })
        } else {
            ProdlogData { entries: Vec::new() }
        };

        // Add new entry
        let host = &capture.host;
        let cmd_long = &capture.cmd;
        let log_filename = get_output_log_filename(&capture);
        let duration_ms = end_time.signed_duration_since(capture.start_time).num_milliseconds() as u64;
        prodlog_data.entries.push(ProdlogEntry {
            host: host.to_string(),
            start_time: capture.start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            duration_ms,
            command: cmd_long.to_string(),
            log_filename: log_filename.to_string(),
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
            exit_code,
        });

        // Write updated JSON file
        fs::write(&json_path, serde_json::to_string_pretty(&prodlog_data)?)?;

        Ok(())
    }
}
