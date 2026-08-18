//! Random master seed. Never derived from a password.

use crate::derive::{digest, expand};
use crate::device::DeviceAuthority;
use core::fmt;
use ed25519_dalek::SigningKey;
use reedhold_core::{DomainTag, Error, IdentityId, Result};

/// 256-bit account root. Losing it without a recovery vault loses the account.
pub struct MasterSeed([u8; 32]);

impl MasterSeed {
    /// Draw a new seed from the operating-system CSPRNG.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Entropy`] when the OS RNG is unavailable.
    pub fn generate() -> Result<Self> {
        let mut bytes = [0_u8; 32];
        getrandom::getrandom(&mut bytes).map_err(|_| Error::Entropy)?;
        Ok(Self(bytes))
    }

    /// Wrap an existing 32-byte seed.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow the raw seed. Callers must not log this.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Derive the stable identity and the device-authorization root.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when HKDF expansion fails.
    pub fn unlock(&self) -> Result<IdentityBundle> {
        let identity_secret = expand(&self.0, DomainTag::IdentityRoot)?;
        let signing = SigningKey::from_bytes(&identity_secret);
        let public = signing.verifying_key().to_bytes();
        let identity = IdentityId::from_digest(digest(DomainTag::Identity, &public));
        let devices = DeviceAuthority::derive(&self.0)?;
        Ok(IdentityBundle {
            identity,
            root_public: public,
            devices,
        })
    }
}

impl fmt::Debug for MasterSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MasterSeed([redacted])")
    }
}

/// Public account material derived from a seed. Contains no master secret.
#[derive(Clone, Debug)]
pub struct IdentityBundle {
    /// Permanent network identifier.
    pub identity: IdentityId,
    /// Ed25519 public key of the identity root.
    pub root_public: [u8; 32],
    /// Device authorization derived from the same seed.
    pub devices: DeviceAuthority,
}

#[cfg(test)]
mod tests {
    use super::MasterSeed;

    #[test]
    fn same_seed_same_identity() {
        let seed = MasterSeed::from_bytes([3_u8; 32]);
        let first = seed.unlock().unwrap();
        let second = seed.unlock().unwrap();
        assert_eq!(first.identity, second.identity);
        assert!(first.identity.to_uri().starts_with("reedhold:identity:"));
    }

    #[test]
    fn different_seeds_diverge() {
        let left = MasterSeed::from_bytes([1_u8; 32]).unlock().unwrap();
        let right = MasterSeed::from_bytes([2_u8; 32]).unlock().unwrap();
        assert_ne!(left.identity, right.identity);
    }
}
