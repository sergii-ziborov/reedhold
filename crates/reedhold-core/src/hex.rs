//! Lower-case hex. Used by identifiers and the host API.

use crate::{Error, Result};

const TABLE: &[u8; 16] = b"0123456789abcdef";

/// Encode bytes as lower-case hex.
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(TABLE[(byte >> 4) as usize] as char);
        out.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Decode a lower- or mixed-case hex string.
///
/// # Errors
///
/// Returns [`Error::Codec`] when the string is odd-length or not hex.
pub fn decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(Error::Codec("hex string has odd length"));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let high = nibble(bytes[index])?;
        let low = nibble(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Ok(out)
}

/// Decode exactly 32 bytes.
///
/// # Errors
///
/// Returns [`Error::Codec`] when the string is not 64 hex characters.
pub fn decode32(hex: &str) -> Result<[u8; 32]> {
    let bytes = decode(hex)?;
    bytes
        .try_into()
        .map_err(|_| Error::Codec("expected 32 hex bytes"))
}

fn nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::Codec("invalid hex digit")),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode32, encode};

    #[test]
    fn round_trip() {
        let raw = [0xab_u8; 32];
        assert_eq!(decode32(&encode(&raw)).unwrap(), raw);
    }
}
