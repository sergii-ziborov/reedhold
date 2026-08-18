//! Offline genesis advertising root.

use crate::certificate::AdOperatorCertificate;
use crate::limits::AdvertisingLimits;
use ed25519_dalek::SigningKey;
use reedhold_core::{Digest32, DomainTag};
use sha2::{Digest, Sha256};

/// Founder advertising key. Keep it offline. Losing it does not stop the mesh.
pub struct AdvertisingRoot {
    public: [u8; 32],
    signing: SigningKey,
}

impl AdvertisingRoot {
    /// Derive from a dedicated 32-byte seed. Never reuse the user `MasterSeed`.
    #[must_use]
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(DomainTag::AdvertisingRoot.as_bytes());
        hasher.update(seed);
        let secret: [u8; 32] = hasher.finalize().into();
        let signing = SigningKey::from_bytes(&secret);
        Self {
            public: signing.verifying_key().to_bytes(),
            signing,
        }
    }

    /// Public key that genesis records. Safe to publish.
    #[must_use]
    pub const fn public_key(&self) -> [u8; 32] {
        self.public
    }

    /// Content-addressed id of the public key.
    #[must_use]
    pub fn public_id(&self) -> Digest32 {
        let mut hasher = Sha256::new();
        hasher.update(DomainTag::AdvertisingRoot.as_bytes());
        hasher.update(self.public);
        Digest32::from_bytes(hasher.finalize().into())
    }

    /// Protocol bounds for this key.
    #[must_use]
    pub const fn limits(&self) -> AdvertisingLimits {
        AdvertisingLimits::GENESIS
    }

    /// Issue a short-lived operator certificate.
    #[must_use]
    pub fn issue_operator(
        &self,
        operator_public: [u8; 32],
        valid_from: u64,
        valid_until: u64,
        max_budget: u64,
    ) -> AdOperatorCertificate {
        AdOperatorCertificate::issue(
            &self.signing,
            self.public,
            operator_public,
            valid_from,
            valid_until,
            max_budget,
        )
    }

    /// Sign a user event? Never. The type system has no such method.
    #[must_use]
    pub const fn can_sign_user_event(&self) -> bool {
        false
    }
}

impl core::fmt::Debug for AdvertisingRoot {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("AdvertisingRoot")
            .field("public", &reedhold_core::encode_hex(&self.public))
            .finish_non_exhaustive()
    }
}
