//! Fixed-size cryptographic identifiers.

use core::fmt;

/// 32-byte digest used as an opaque identifier.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digest32(pub(crate) [u8; 32]);

impl Digest32 {
    /// Wrap an already-computed digest.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow the raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lower-case hex encoding.
    #[must_use]
    pub fn to_hex(self) -> String {
        crate::hex::encode(&self.0)
    }

    /// Parse 64 hex characters.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Codec`] when the string is not 32 hex bytes.
    pub fn from_hex(hex: &str) -> crate::Result<Self> {
        Ok(Self(crate::hex::decode32(hex)?))
    }
}

impl fmt::Debug for Digest32 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Digest32({})", self.to_hex())
    }
}

/// Permanent network identity. Independent of username and password.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdentityId(Digest32);

impl IdentityId {
    /// Construct from a digest.
    #[must_use]
    pub const fn from_digest(digest: Digest32) -> Self {
        Self(digest)
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }

    /// Protocol URI: `reedhold:identity:<hex>`.
    #[must_use]
    pub fn to_uri(self) -> String {
        format!("reedhold:identity:{}", self.0.to_hex())
    }
}

/// Authorized device belonging to one identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceId(Digest32);

impl DeviceId {
    /// Construct from a digest.
    #[must_use]
    pub const fn from_digest(digest: Digest32) -> Self {
        Self(digest)
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }

    /// Lower-case hex of the device id.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.to_hex()
    }
}

/// Content-addressed payload identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContentId(Digest32);

impl ContentId {
    /// Construct from a digest.
    #[must_use]
    pub const fn from_digest(digest: Digest32) -> Self {
        Self(digest)
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Digest32;

    #[test]
    fn hex_is_stable() {
        let digest = Digest32::from_bytes([0xab; 32]);
        assert_eq!(digest.to_hex().len(), 64);
        assert!(digest.to_hex().chars().all(|ch| ch.is_ascii_hexdigit()));
        assert_eq!(digest.to_hex(), Digest32::from_bytes([0xab; 32]).to_hex());
    }
}
