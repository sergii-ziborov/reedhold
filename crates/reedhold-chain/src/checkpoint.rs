//! Compact checkpoint view. The chain stores this, not posts.

use crate::header::Header;
use reedhold_core::{Digest32, NetworkId};

/// Merkle-style epoch root. The payload of the chain is this, not posts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    /// Network this checkpoint belongs to.
    pub network: NetworkId,
    /// Monotonic epoch.
    pub epoch: u64,
    /// Combined identity / group / storage / reputation root.
    pub state_root: Digest32,
}

impl Checkpoint {
    /// Construct a checkpoint.
    #[must_use]
    pub const fn new(network: NetworkId, epoch: u64, state_root: Digest32) -> Self {
        Self {
            network,
            epoch,
            state_root,
        }
    }

    /// View of a compact header.
    #[must_use]
    pub fn from_header(header: &Header) -> Self {
        Self {
            network: header.network,
            epoch: header.epoch,
            state_root: header.state_root(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Checkpoint;
    use crate::header::Header;
    use reedhold_core::NetworkId;

    #[test]
    fn checkpoint_does_not_carry_payload_bytes() {
        let header = Header::genesis(NetworkId::DEV);
        let checkpoint = Checkpoint::from_header(&header);
        assert_eq!(checkpoint.epoch, 0);
        assert_eq!(checkpoint.state_root, header.state_root());
        assert!(!header.encode().windows(2).any(|window| window == b"dm"));
    }
}
