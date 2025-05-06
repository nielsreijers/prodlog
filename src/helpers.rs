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
