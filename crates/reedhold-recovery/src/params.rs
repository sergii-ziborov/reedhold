//! Argon2id parameters. Test builds use a cheap profile on purpose.

/// Memory-hard KDF settings stored next to the sealed seed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub memory_kib: u32,
    /// Pass count.
    pub iterations: u32,
    /// Parallelism.
    pub parallelism: u32,
}

impl KdfParams {
    /// Cheap profile for unit tests. Not a production recommendation.
    pub const TEST: Self = Self {
        memory_kib: 8 * 1024,
        iterations: 1,
        parallelism: 1,
    };

    /// Encode as three little-endian `u32` values.
    #[must_use]
    pub fn to_bytes(self) -> [u8; 12] {
        let mut out = [0_u8; 12];
        out[..4].copy_from_slice(&self.memory_kib.to_le_bytes());
        out[4..8].copy_from_slice(&self.iterations.to_le_bytes());
        out[8..].copy_from_slice(&self.parallelism.to_le_bytes());
        out
    }

    /// Decode three little-endian `u32` values.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 12]) -> Self {
        Self {
            memory_kib: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            iterations: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            parallelism: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        }
    }
}
