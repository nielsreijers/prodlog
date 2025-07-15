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

pub fn unescape_and_unquote_cmd(raw_cmd: &str) -> String {
    let mut result = Vec::new();
    let mut current_element = String::new();
    let mut escaped = false;

    let mut chars = raw_cmd.chars().peekable();
    while let Some(ch) = chars.next() {
        if escaped {
            // Previous character was a backslash: unescape this character
            current_element.push(ch);
            escaped = false;
        } else if ch == '\\' {
            // This is an escape character: unescape the next character
            escaped = true;
        } else if ch == ' ' {
            // End of element since this space was not escaped
            result.push(clean_element(&current_element));
            current_element.clear();
        } else {
            // Regular character
            current_element.push(ch);
        }
    }

    // Add the last element if not empty
    if !current_element.is_empty() {
        result.push(clean_element(&current_element));
    }

    result.join(" ")
}

fn clean_element(element: &str) -> String {
    // Check if element needs quotes (contains special characters)
    let unquoted = unquote_element(element);

    let needs_quotes = unquoted.chars().any(|c| {
        matches!(c, '$' | ' ' | '\t' | '\n' | '!' | '*' | '?' | '[' | ']' | '{' | '}' | '(' | ')' | ';' | '&' | '|' | '<' | '>' | '`' | '~' | '#' | '\'' | '"' | '\\')
    });

    if needs_quotes {
        // Keep the element as-is since it contains special characters
        element.to_string()
    } else {
        unquoted
    }
}

fn unquote_element(element: &str) -> String {
    if (element.starts_with('"') && element.ends_with('"') && element.len() > 1)
       || (element.starts_with('\'') && element.ends_with('\'') && element.len() > 1) {
        element[1..element.len()-1].to_string()
    } else {
        element.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_and_unquote_cmd_simple() {
        // The command received from the prodlog script will have every element quoted and spaces and backslashes escaped.
        // This function unescapes the command and unquotes the elements if the do not contain special characters.
        assert_eq!(unescape_and_unquote_cmd("'ls' '-l'"), "ls -l");
        assert_eq!(unescape_and_unquote_cmd("'echo' 'hello\\ world'"), "echo 'hello world'");
        assert_eq!(unescape_and_unquote_cmd("'echo' '\\\\'"), "echo '\\'");
    }
}
