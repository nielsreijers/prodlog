use std::path::PathBuf;
use std::fs;
use crate::model::*;
use crate::prodlog_panic;
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


// TMP pub only while UI reads from JSON and not yet from SQLite
pub fn read_prodlog_data(json_path: &PathBuf) -> Result<ProdlogDataV2_3, std::io::Error> {
    if let Ok(content) = fs::read_to_string(&json_path) {
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(data);
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_2_to_v2_3(data));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_2_to_v2_3(v2_1_to_v2_2(data)));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_2_to_v2_3(v2_1_to_v2_2(v2_0_to_v2_1(data))));
        }
        prodlog_panic(&format!("Failed to read prodlog data from {}", json_path.display()));
    } else {
        Ok(ProdlogDataV2_3 {
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
            entries: vec![],
        })
    }
}

impl Sink for JsonSink {
    fn add_entry(&mut self, capture: &CaptureV2_3) -> Result<(), std::io::Error> {
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
    fn get_entries(&self, filters: &super::Filters) -> Result<Vec<CaptureV2_3>, std::io::Error> {
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

            if let Some(true) = &filters.show_noop {
                // Don't filter out no-op entries
            } else {
                if entry.is_noop {
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

    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_3>, std::io::Error> {
        let data = read_prodlog_data(&self.prodlog_file)?;
        let entry = data.entries.into_iter().find(|e| e.uuid == uuid);
        Ok(entry)
    }
}