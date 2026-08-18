//! Canonical writer.

/// Append-only canonical encoder.
#[derive(Clone, Debug, Default)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    /// Empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Write a single byte tag.
    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    /// Write a little-endian `u16`.
    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Write a little-endian `u64`.
    pub fn write_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Write a 32-byte digest.
    pub fn write_digest32(&mut self, value: &[u8; 32]) {
        self.bytes.extend_from_slice(value);
    }

    /// Write a length-prefixed byte string. Length is a little-endian `u32`.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the slice is longer than
    /// `u32::MAX`.
    pub fn write_bytes(&mut self, value: &[u8]) -> reedhold_core::Result<()> {
        let len = u32::try_from(value.len())
            .map_err(|_| reedhold_core::Error::Codec("byte string exceeds u32 length"))?;
        self.bytes.extend_from_slice(&len.to_le_bytes());
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    /// Consume the writer and return the encoded bytes.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;

    #[test]
    fn encoding_is_deterministic() {
        let mut first = Writer::new();
        first.write_u8(7);
        first.write_u16(0x1234);
        first.write_bytes(b"reed").unwrap();
        let mut second = Writer::new();
        second.write_u8(7);
        second.write_u16(0x1234);
        second.write_bytes(b"reed").unwrap();
        assert_eq!(first.finish(), second.finish());
    }
}
