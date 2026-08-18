//! Random master seed. Never derived from a password.

use crate::device::DeviceAuthority;
use crate::root::IdentityRoot;
use core::fmt;
use reedhold_core::{Error, Result};

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
        let root = IdentityRoot::derive(&self.0)?;
        let devices = DeviceAuthority::derive(&self.0)?;
        Ok(IdentityBundle {
            identity: root.identity,
            root,
            devices,
        })
    }
}

impl fmt::Debug for MasterSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MasterSeed([redacted])")
    }
}

/// Derived account material. Holds the identity-root key, not the master seed.
#[derive(Clone, Debug)]
pub struct IdentityBundle {
    /// Permanent network identifier.
    pub identity: reedhold_core::IdentityId,
    /// Root signer used for device grants.
    pub root: IdentityRoot,
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
