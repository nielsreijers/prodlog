use base64::Engine as _;

pub fn base64_decode(data: &str) -> String {
    use base64::{Engine as _, engine::{general_purpose}};
    match general_purpose::STANDARD.decode(data) {
        Ok(bytes) => String::from_utf8(bytes).unwrap(),
        Err(e) => {
            println!("Error decoding base64: {}", e);
            data.to_string() // Shouldn't happen, but if it does, just return the original string.
        }
    }
}

pub fn base64_encode(data: &Vec<u8>) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}