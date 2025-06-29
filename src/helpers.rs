use crate::model::CaptureV2_4;

pub fn base64_decode_string(data: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    match general_purpose::STANDARD.decode(data) {
        Ok(bytes) => String::from_utf8(bytes).unwrap(),
        Err(e) => {
            println!("Error decoding base64: {}", e);
            data.to_string() // Shouldn't happen, but if it does, just return the original string.
        }
    }
}

pub fn base64_decode(data: &str) -> Vec<u8> {
    use base64::{Engine as _, engine::general_purpose};
    match general_purpose::STANDARD.decode(data) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Error decoding base64: {}", e);
            data.as_bytes().to_vec() // Shouldn't happen, but if it does, just return the original string.
        }
    }
}

pub fn compare_major_minor_versions(version1: &str, version2: &str) -> bool {
    let v1_parts: Vec<&str> = version1.split('.').collect();
    let v2_parts: Vec<&str> = version2.split('.').collect();
    
    if v1_parts.len() < 2 || v2_parts.len() < 2 {
        return false;
    }
    
    v1_parts[0] == v2_parts[0] && v1_parts[1] == v2_parts[1]
}

/// Redacts passwords from all fields of an entry
/// Returns true if any redaction occurred
pub fn redact_passwords_from_entry(entry: &mut CaptureV2_4, passwords: &[String]) -> bool {
    let mut redacted = false;

    // Redact passwords in command
    for password in passwords {
        if entry.cmd.contains(password) {
            entry.cmd = entry.cmd.replace(password, "[REDACTED]");
            redacted = true;
        }
    }

    // Redact passwords in captured output
    let output_str = String::from_utf8_lossy(&entry.captured_output);
    let mut new_output = output_str.to_string();
    let mut output_modified = false;
    for password in passwords {
        if new_output.contains(password) {
            new_output = new_output.replace(password, "[REDACTED]");
            output_modified = true;
            redacted = true;
        }
    }
    if output_modified {
        entry.captured_output = new_output.into_bytes();
    }

    // Redact passwords in original content (for edit entries)
    if !entry.original_content.is_empty() {
        let original_str = String::from_utf8_lossy(&entry.original_content);
        let mut new_original = original_str.to_string();
        let mut original_modified = false;
        for password in passwords {
            if new_original.contains(password) {
                new_original = new_original.replace(password, "[REDACTED]");
                original_modified = true;
                redacted = true;
            }
        }
        if original_modified {
            entry.original_content = new_original.into_bytes();
        }
    }

    // Redact passwords in edited content (for edit entries)
    if !entry.edited_content.is_empty() {
        let edited_str = String::from_utf8_lossy(&entry.edited_content);
        let mut new_edited = edited_str.to_string();
        let mut edited_modified = false;
        for password in passwords {
            if new_edited.contains(password) {
                new_edited = new_edited.replace(password, "[REDACTED]");
                edited_modified = true;
                redacted = true;
            }
        }
        if edited_modified {
            entry.edited_content = new_edited.into_bytes();
        }
    }

    redacted
}
