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
pub fn read_prodlog_data(json_path: &PathBuf) -> Result<ProdlogDataV2_4, std::io::Error> {
    if let Ok(content) = fs::read_to_string(&json_path) {
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(data);
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_3_to_v2_4(data));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_3_to_v2_4(v2_2_to_v2_3(data)));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_3_to_v2_4(v2_2_to_v2_3(v2_1_to_v2_2(data))));
        }
        if let Ok(data) = serde_json::from_str(&content) {
            return Ok(v2_3_to_v2_4(v2_2_to_v2_3(v2_1_to_v2_2(v2_0_to_v2_1(data)))));
        }
        prodlog_panic(&format!("Failed to read prodlog data from {}", json_path.display()));
    } else {
        Ok(ProdlogDataV2_4 {
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
            entries: vec![],
        })
    }
}

impl Sink for JsonSink {
    fn add_entry(&mut self, capture: &CaptureV2_4) -> Result<(), std::io::Error> {
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
    fn get_entries(&self, filters: &super::Filters) -> Result<Vec<CaptureV2_4>, std::io::Error> {
        let data = read_prodlog_data(&self.prodlog_file)?;
            let mut filtered_entries = Vec::new();
        
        for entry in data.entries.into_iter() {
            // Apply date range filters
            if let Some(date_from) = &filters.date_from {
                let from_time = format!("{}T00:00:00Z", date_from);
                if entry.start_time.to_rfc3339() < from_time {
                    continue;
                }
            }
            
            if let Some(date_to) = &filters.date_to {
                let to_time = format!("{}T23:59:59Z", date_to);
                if entry.start_time.to_rfc3339() > to_time {
                    continue;
                }
            }
            
            if let Some(host) = &filters.host {
                if !entry.host.to_lowercase().contains(&host.to_lowercase()) {
                    continue;
                }
            }
            
            if let Some(command) = &filters.search {
                if !entry.cmd.to_lowercase().contains(&command.to_lowercase()) && 
                   !entry.message.to_lowercase().contains(&command.to_lowercase()) {
                    continue;
                }
            }
            
            if let Some(content) = &filters.search_content {
                let search_term = content.to_lowercase();
                let mut found = false;
                
                // Search in cmd and message
                if entry.cmd.to_lowercase().contains(&search_term) || 
                   entry.message.to_lowercase().contains(&search_term) {
                    found = true;
                }
                
                // Search in captured output
                if !found {
                    if let Ok(output_text) = String::from_utf8(entry.captured_output.clone()) {
                        if output_text.to_lowercase().contains(&search_term) {
                            found = true;
                        }
                    }
                }
                
                // Search in original content
                if !found {
                    if let Ok(orig_text) = String::from_utf8(entry.original_content.clone()) {
                        if orig_text.to_lowercase().contains(&search_term) {
                            found = true;
                        }
                    }
                }
                
                // Search in edited content
                if !found {
                    if let Ok(edited_text) = String::from_utf8(entry.edited_content.clone()) {
                        if edited_text.to_lowercase().contains(&search_term) {
                            found = true;
                        }
                    }
                }
                
                if !found {
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
            
            filtered_entries.push(entry);
        }

        Ok(filtered_entries)
    }

    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_4>, std::io::Error> {
        let data = read_prodlog_data(&self.prodlog_file)?;
        let entry = data.entries.into_iter().find(|e| e.uuid == uuid);
        Ok(entry)
    }
}