//! High-reputation identities may propose distribution. They never push to a user id.

use crate::bucket::{DISTRIBUTOR_MIN_BUCKET, bucket};
use crate::math::isqrt;
use reedhold_core::Digest32;

/// Protocol cap on slots one distributor may offer per epoch.
pub const CAPACITY_CAP: u32 = 64;

/// An eligible distributor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Distributor {
    /// Identity digest.
    pub id: Digest32,
    /// Host-supplied mature strength 0..=10000.
    pub strength: u32,
    /// Quality milli. Starts at 1000; hides lower it.
    pub quality: u32,
}

impl Distributor {
    /// Eligible if the strength bucket is at least B2.
    #[must_use]
    pub fn eligible(strength: u32) -> bool {
        bucket(strength) >= DISTRIBUTOR_MIN_BUCKET
    }

    /// `C0 * sqrt(strength) * quality / 1000`, capped.
    #[must_use]
    pub fn capacity(&self) -> u32 {
        let raw = isqrt(self.strength).saturating_mul(self.quality) / 1000;
        raw.clamp(1, CAPACITY_CAP)
    }
}

#[cfg(test)]
mod tests {
    use super::Distributor;
    use reedhold_core::Digest32;

    #[test]
    fn weak_accounts_cannot_distribute() {
        assert!(!Distributor::eligible(100));
        assert!(Distributor::eligible(3000));
        let strong = Distributor {
            id: Digest32::from_bytes([1; 32]),
            strength: 8000,
            quality: 1000,
        };
        let weak = Distributor {
            id: Digest32::from_bytes([2; 32]),
            strength: 3000,
            quality: 1000,
        };
        assert!(strong.capacity() > weak.capacity());
    }
}
