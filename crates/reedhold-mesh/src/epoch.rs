//! Sync epochs. One day on the clock, one lottery on the mesh.

use reedhold_core::{Digest32, DomainTag, NetworkId};
use sha2::{Digest, Sha256};

/// Length of a sync epoch in seconds. Relays are redrawn each epoch.
pub const EPOCH_SECONDS: u64 = 86_400;

/// A numbered sync window. Index 0 is 1970-01-01 UTC.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyncEpoch {
    /// Whole days since the Unix epoch.
    pub index: u64,
}

impl SyncEpoch {
    /// Convert a Unix timestamp into the current sync epoch.
    #[must_use]
    pub const fn from_unix_secs(unix_secs: u64) -> Self {
        Self {
            index: unix_secs / EPOCH_SECONDS,
        }
    }

    /// Next epoch. Yesterday's relays are no longer preferred.
    #[must_use]
    pub const fn next(self) -> Self {
        Self {
            index: self.index.saturating_add(1),
        }
    }
}

/// Shared lottery seed for one epoch.
///
/// `prior_commit` is chain/commit randomness later. Until that exists, hosts
/// pass the previous epoch's observed head. A censor who does not know the
/// prior commit cannot pre-block tomorrow's relays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EpochSeed(Digest32);

impl EpochSeed {
    /// Derive the seed every honest node will compute for this epoch.
    #[must_use]
    pub fn derive(network: NetworkId, epoch: SyncEpoch, prior_commit: &[u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(DomainTag::SyncEpoch.as_bytes());
        hasher.update(network.as_str().as_bytes());
        hasher.update(epoch.index.to_le_bytes());
        hasher.update(prior_commit);
        Self(Digest32::from_bytes(hasher.finalize().into()))
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{EpochSeed, SyncEpoch};
    use reedhold_core::NetworkId;

    #[test]
    fn adjacent_days_are_different_epochs() {
        let morning = SyncEpoch::from_unix_secs(86_400);
        let next_morning = SyncEpoch::from_unix_secs(86_400 * 2);
        assert_eq!(morning.next(), next_morning);
        assert_ne!(morning, next_morning);
    }

    #[test]
    fn seed_moves_when_the_prior_commit_moves() {
        let epoch = SyncEpoch { index: 10 };
        let a = EpochSeed::derive(NetworkId::DEV, epoch, &[1_u8; 32]);
        let b = EpochSeed::derive(NetworkId::DEV, epoch, &[2_u8; 32]);
        assert_ne!(a, b);
    }
}
