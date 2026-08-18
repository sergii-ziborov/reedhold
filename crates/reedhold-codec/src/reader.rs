//! Canonical reader.

use reedhold_core::{Error, Result};

/// Sequential decoder over a borrowed buffer.
#[derive(Clone, Debug)]
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Start reading `bytes`.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// Read one byte.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the buffer is empty.
    pub fn read_u8(&mut self) -> Result<u8> {
        let (head, rest) = self
            .rest
            .split_first()
            .ok_or(Error::Codec("truncated u8"))?;
        self.rest = rest;
        Ok(*head)
    }

    /// Read a little-endian `u16`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when fewer than two bytes remain.
    pub fn read_u16(&mut self) -> Result<u16> {
        let raw = self.take(2)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    /// Read a little-endian `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when fewer than eight bytes remain.
    pub fn read_u64(&mut self) -> Result<u64> {
        let raw = self.take(8)?;
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(raw);
        Ok(u64::from_le_bytes(bytes))
    }

    /// Read a 32-byte digest.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when fewer than 32 bytes remain.
    pub fn read_digest32(&mut self) -> Result<[u8; 32]> {
        let raw = self.take(32)?;
        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(raw);
        Ok(bytes)
    }

    /// Read a length-prefixed byte string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the length prefix or payload is truncated.
    pub fn read_bytes(&mut self) -> Result<&'a [u8]> {
        let raw = self.take(4)?;
        let len = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
        self.take(len)
    }

    /// Fail unless the buffer is exhausted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when unread trailing bytes remain.
    pub fn finish(self) -> Result<()> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(Error::Codec("trailing bytes"))
        }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.rest.len() < count {
            return Err(Error::Codec("truncated buffer"));
        }
        let (head, rest) = self.rest.split_at(count);
        self.rest = rest;
        Ok(head)
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;
    use crate::Writer;

    #[test]
    fn round_trip() {
        let mut writer = Writer::new();
        writer.write_u8(3);
        writer.write_u64(99);
        writer.write_bytes(b"hold").unwrap();
        let encoded = writer.finish();
        let mut reader = Reader::new(&encoded);
        assert_eq!(reader.read_u8().unwrap(), 3);
        assert_eq!(reader.read_u64().unwrap(), 99);
        assert_eq!(reader.read_bytes().unwrap(), b"hold");
        reader.finish().unwrap();
    }
}
