//! Identity root signing key. Separate from device logs.

use crate::derive::{digest, expand};
use ed25519_dalek::{Signature, Signer, SigningKey};
use reedhold_core::{DomainTag, IdentityId, Result};

/// Root signing authority for one account.
#[derive(Clone)]
pub struct IdentityRoot {
    /// Permanent identity.
    pub identity: IdentityId,
    /// Ed25519 public key.
    pub public: [u8; 32],
    signing: SigningKey,
}

impl IdentityRoot {
    pub(crate) fn derive(master: &[u8; 32]) -> Result<Self> {
        let secret = expand(master, DomainTag::IdentityRoot)?;
        let signing = SigningKey::from_bytes(&secret);
        let public = signing.verifying_key().to_bytes();
        Ok(Self {
            identity: IdentityId::from_digest(digest(DomainTag::Identity, &public)),
            public,
            signing,
        })
    }

    /// Sign an authorization body.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing.sign(message)
    }
}

impl core::fmt::Debug for IdentityRoot {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("IdentityRoot")
            .field("identity", &self.identity)
            .field("public", &reedhold_core::encode_hex(&self.public))
            .finish_non_exhaustive()
    }
}
