use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::helpers;
use crate::prodlog_panic;
use crate::model::{CaptureType, CaptureV2_2};
use super::{Sink, UiSource};
use uuid::Uuid;

pub struct JsonSink {
    prodlog_file: PathBuf,
}

impl JsonSink {
    pub fn new(prodlog_file: &PathBuf) -> Self {
        // Check the file is valid so we don't crash while logging a command if it's not
        read_prodlog_data(prodlog_file).unwrap();
        let prodlog_file = prodlog_file.clone();
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
// TMP pub only while UI reads from JSON and not yet from SQLite
pub struct ProdlogDataV2_2 {
    prodlog_version: String,
    pub entries: Vec<CaptureV2_2>,
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

// TMP pub only while UI reads from JSON and not yet from SQLite
pub fn read_prodlog_data(json_path: &PathBuf) -> Result<ProdlogDataV2_2, std::io::Error> {
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
        prodlog_panic(&format!("Failed to read prodlog data from {}", json_path.display()));
    } else {
        Ok(ProdlogDataV2_2 {
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
            entries: vec![],
        })
    }
}

impl Sink for JsonSink {
    fn add_entry(&mut self, capture: &CaptureV2_2) -> Result<(), std::io::Error> {
        // Read existing JSON file
        let mut prodlog_data = read_prodlog_data(&self.prodlog_file)?;
        
        // Find and remove any existing entry with the same UUID
        prodlog_data.entries.retain(|entry| entry.uuid != capture.uuid);
        
        // Add the new/updated entry
        prodlog_data.entries.push(capture.clone());

        // Write updated JSON file
        fs::write(&self.prodlog_file, serde_json::to_string_pretty(&prodlog_data)?)?;
        Ok(())
    }
}

impl UiSource for JsonSink {
    fn get_entries(&self, filters: &super::Filters) -> Result<Vec<CaptureV2_2>, std::io::Error> {
        let data = read_prodlog_data(&self.prodlog_file)?;
            let mut filtered_entries = Vec::new();
        
        for entry in data.entries.into_iter() {
            // Apply date, host, and command filters
            if let Some(date) = &filters.date {
                if !entry.start_time.to_rfc3339().starts_with(date) {
                    continue;
                }
            }
            
            if let Some(host) = &filters.host {
                if !entry.host.to_lowercase().contains(&host.to_lowercase()) {
                    continue;
                }
            }
            
            if let Some(command) = &filters.command {
                if !entry.cmd.to_lowercase().contains(&command.to_lowercase()) {
                    continue;
                }
            }
    
            // Check output content if output filter is present
            if let Some(output_filter) = &filters.output {
                if !output_filter.is_empty() {
                    let output_content = entry.output_as_string();
                    if !output_content.to_lowercase().contains(&output_filter.to_lowercase()) {
                        continue;
                    }
                }
            }
            
            filtered_entries.push(entry);
        }

        Ok(filtered_entries)
    }

    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_2>, std::io::Error> {
        let data = read_prodlog_data(&self.prodlog_file)?;
        let entry = data.entries.into_iter().find(|e| e.uuid == uuid);
        Ok(entry)
    }
}