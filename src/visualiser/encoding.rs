//! Base64 encoding utilities for VTK binary data.
//!
//! Provides a minimal base64 encoder for VTK XML binary format.
//! This is an internal implementation detail and not exposed in the public API.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode binary data to base64 string.
///
/// This is a minimal implementation sufficient for VTK binary data encoding.
pub(crate) fn encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i + 2 < data.len() {
        let b1 = data[i];
        let b2 = data[i + 1];
        let b3 = data[i + 2];

        result.push(ALPHABET[(b1 >> 2) as usize] as char);
        result.push(ALPHABET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(ALPHABET[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char);
        result.push(ALPHABET[(b3 & 0x3f) as usize] as char);

        i += 3;
    }

    // Handle remaining bytes
    match data.len() - i {
        1 => {
            let b1 = data[i];
            result.push(ALPHABET[(b1 >> 2) as usize] as char);
            result.push(ALPHABET[((b1 & 0x03) << 4) as usize] as char);
            result.push('=');
            result.push('=');
        }
        2 => {
            let b1 = data[i];
            let b2 = data[i + 1];
            result.push(ALPHABET[(b1 >> 2) as usize] as char);
            result.push(ALPHABET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
            result.push(ALPHABET[((b2 & 0x0f) << 2) as usize] as char);
            result.push('=');
        }
        _ => {}
    }

    result
}
