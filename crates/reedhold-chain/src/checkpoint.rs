//! One compact epoch checkpoint.

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
}

#[cfg(test)]
mod tests {
    use super::Checkpoint;
    use reedhold_core::{Digest32, NetworkId};

    #[test]
    fn checkpoint_does_not_carry_payload_bytes() {
        let root = Digest32::from_bytes([0; 32]);
        let checkpoint = Checkpoint::new(NetworkId::DEV, 1, root);
        assert_eq!(checkpoint.epoch, 1);
        assert_eq!(checkpoint.state_root, root);
    }
}
