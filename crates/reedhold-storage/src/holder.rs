//! Storage node identity. Independent of mesh `PeerId`.

use reedhold_core::{Digest32, Result};

/// Who holds a shard. Mapped to a mesh peer by the host.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HolderId(Digest32);

impl HolderId {
    /// Wrap a digest.
    #[must_use]
    pub const fn from_digest(digest: Digest32) -> Self {
        Self(digest)
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }

    /// Hex form for the host API.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.to_hex()
    }

    /// Parse 32 hex bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the string is not 32 bytes.
    pub fn from_hex(hex: &str) -> Result<Self> {
        Ok(Self(Digest32::from_hex(hex)?))
    }
}
