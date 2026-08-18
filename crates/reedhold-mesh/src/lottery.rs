//! Deterministic relay lottery. Ordinary peers, not a server caste.

use crate::epoch::EpochSeed;
use crate::ports::PeerId;
use reedhold_core::DomainTag;
use sha2::{Digest, Sha256};

/// Rank one peer for this epoch. Lower digest wins (stable, not "weaker").
#[must_use]
pub fn relay_score(seed: &EpochSeed, peer: PeerId) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::RelayScore.as_bytes());
    hasher.update(seed.as_digest().as_bytes());
    hasher.update(peer.as_digest().as_bytes());
    hasher.finalize().into()
}

/// Pick up to `limit` transitional relays from `candidates`.
///
/// The company host, if present in the list, is ignored here. It is optional
/// and must not win a lottery slot that would look like a requirement.
#[must_use]
pub fn select_relays(seed: &EpochSeed, candidates: &[PeerId], limit: usize) -> Vec<PeerId> {
    let mut scored: Vec<([u8; 32], PeerId)> = candidates
        .iter()
        .copied()
        .map(|peer| (relay_score(seed, peer), peer))
        .collect();
    scored.sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    scored.dedup_by_key(|entry| entry.1);
    scored
        .into_iter()
        .take(limit)
        .map(|(_, peer)| peer)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::select_relays;
    use crate::epoch::{EpochSeed, SyncEpoch};
    use crate::ports::PeerId;
    use reedhold_core::{Digest32, NetworkId};

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn same_seed_same_roster() {
        let seed = EpochSeed::derive(NetworkId::DEV, SyncEpoch { index: 3 }, &[0_u8; 32]);
        let candidates: Vec<PeerId> = (1_u8..=20).map(peer).collect();
        let first = select_relays(&seed, &candidates, 5);
        let second = select_relays(&seed, &candidates, 5);
        assert_eq!(first, second);
        assert_eq!(first.len(), 5);
    }
}
