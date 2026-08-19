//! Epoch subtree roots. Combined into one state root. Never message bytes.

use crate::hash::digest;
use reedhold_core::{Digest32, DomainTag};

/// Compact commitments for one epoch. Each field is a Merkle root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EpochRoots {
    /// Identity heads.
    pub identity: Digest32,
    /// Group heads.
    pub groups: Digest32,
    /// Durable storage contracts.
    pub storage: Digest32,
    /// Reputation epoch (zeros until stage 7).
    pub reputation: Digest32,
    /// Advertising market (zeros until stage 8).
    pub ads: Digest32,
}

impl EpochRoots {
    /// All-zero roots. Genesis and unused subtrees.
    #[must_use]
    pub const fn empty() -> Self {
        let zero = Digest32::from_bytes([0; 32]);
        Self {
            identity: zero,
            groups: zero,
            storage: zero,
            reputation: zero,
            ads: zero,
        }
    }

    /// `H(chain-state || identity || groups || storage || reputation || ads)`.
    #[must_use]
    pub fn state_root(self) -> Digest32 {
        digest(
            DomainTag::ChainState,
            &[
                self.identity.as_bytes(),
                self.groups.as_bytes(),
                self.storage.as_bytes(),
                self.reputation.as_bytes(),
                self.ads.as_bytes(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::EpochRoots;
    use reedhold_core::Digest32;

    #[test]
    fn different_subtrees_change_the_state_root() {
        let mut roots = EpochRoots::empty();
        let empty = roots.state_root();
        roots.identity = Digest32::from_bytes([1; 32]);
        assert_ne!(roots.state_root(), empty);
    }
}
