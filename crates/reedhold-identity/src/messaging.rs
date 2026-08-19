//! Static X25519 messaging keys. MLS replaces the group schedule later.

use crate::derive::expand;
use hkdf::Hkdf;
use reedhold_core::{DomainTag, Error, IdentityId, Result};
use sha2::Sha256;
use x25519_dalek::{X25519_BASEPOINT_BYTES, x25519};

/// Messaging root derived from `MasterSeed`. Separate from identity signing.
#[derive(Clone)]
pub struct MessagingKeys {
    secret: [u8; 32],
    public: [u8; 32],
}

impl MessagingKeys {
    pub(crate) fn derive(master: &[u8; 32]) -> Result<Self> {
        let secret = expand(master, DomainTag::MessagingRoot)?;
        Ok(Self {
            public: x25519(secret, X25519_BASEPOINT_BYTES),
            secret,
        })
    }

    /// X25519 public key. Safe to publish on a talk packet.
    #[must_use]
    pub const fn public_bytes(&self) -> [u8; 32] {
        self.public
    }

    /// Pairwise conversation key. Both sides compute the same 32 bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when HKDF expansion fails.
    pub fn agree(
        &self,
        peer_public: &[u8; 32],
        local_id: IdentityId,
        peer_id: IdentityId,
    ) -> Result<[u8; 32]> {
        let shared = x25519(self.secret, *peer_public);
        let (lo, hi) = if local_id <= peer_id {
            (local_id, peer_id)
        } else {
            (peer_id, local_id)
        };
        let mut info = DomainTag::TalkPair.as_bytes().to_vec();
        info.extend_from_slice(lo.as_digest().as_bytes());
        info.extend_from_slice(hi.as_digest().as_bytes());
        let hkdf = Hkdf::<Sha256>::new(None, &shared);
        let mut okm = [0_u8; 32];
        hkdf.expand(&info, &mut okm)
            .map_err(|_| Error::Identity("pairwise hkdf failed"))?;
        Ok(okm)
    }

    /// Key that seals the local group book. Not a network key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when HKDF expansion fails.
    pub fn book_key(&self) -> Result<[u8; 32]> {
        let hkdf = Hkdf::<Sha256>::new(None, &self.secret);
        let mut okm = [0_u8; 32];
        hkdf.expand(DomainTag::CircleBook.as_bytes(), &mut okm)
            .map_err(|_| Error::Identity("circle-book hkdf failed"))?;
        Ok(okm)
    }
}

impl core::fmt::Debug for MessagingKeys {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("MessagingKeys")
            .field("public", &reedhold_core::encode_hex(&self.public))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::MessagingKeys;
    use reedhold_core::{Digest32, IdentityId};

    fn id(byte: u8) -> IdentityId {
        IdentityId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn agreement_is_commutative() {
        let alice = MessagingKeys::derive(&[1_u8; 32]).unwrap();
        let bob = MessagingKeys::derive(&[2_u8; 32]).unwrap();
        let left = alice.agree(&bob.public_bytes(), id(1), id(2)).unwrap();
        let right = bob.agree(&alice.public_bytes(), id(2), id(1)).unwrap();
        assert_eq!(left, right);
        assert_ne!(alice.public_bytes(), bob.public_bytes());
    }
}
