//! The one value a client will not take from anywhere else.
//!
//! Everything a node believes about the network is reached by walking back to
//! genesis. If that starting point can be supplied by a config file, a
//! bootstrap peer, or a crafted message, then whoever supplies it decides what
//! the truth is. So it is a constant compiled into the binary, and the test
//! below pins it to the header the code actually builds: change either and the
//! build fails rather than quietly following someone else's chain.
//!
//! This protects a stock client from being redirected. It cannot stop someone
//! from compiling their own binary with a different constant — nothing can,
//! and claiming otherwise would be a lie. What it means is that their build
//! is a different network, visibly, and honest nodes will not follow it.

use crate::fork::ForkChoice;
use reedhold_core::Digest32;

/// Hash of the header this network was born with.
pub const NETWORK_GENESIS: [u8; 32] = [
    0xe9, 0x39, 0xc0, 0xe5, 0xc4, 0xb3, 0xc1, 0xc8, 0x61, 0xa8, 0x05, 0xee, 0x4c, 0x32, 0xf0, 0x1a,
    0xc2, 0x49, 0xf5, 0x43, 0x94, 0x4c, 0x77, 0x13, 0xa3, 0x1e, 0x3c, 0xfb, 0xd5, 0xde, 0x05, 0xcb,
];

/// The genesis every honest node starts from.
#[must_use]
pub fn network_genesis() -> Digest32 {
    Digest32::from_bytes(NETWORK_GENESIS)
}

/// Fork choice anchored at the compiled-in genesis.
///
/// There is deliberately no constructor that takes a genesis from the outside.
#[must_use]
pub fn network_rule() -> ForkChoice {
    ForkChoice::new(network_genesis())
}

#[cfg(test)]
mod tests {
    use super::{network_genesis, network_rule};
    use crate::fork::Branch;
    use crate::header::Header;
    use crate::roots::EpochRoots;
    use reedhold_core::{Digest32, NetworkId};

    #[test]
    fn the_constant_matches_the_header_the_code_builds() {
        assert_eq!(
            Header::genesis(NetworkId::DEV).hash(),
            network_genesis(),
            "genesis drifted: either the header format changed or the anchor \
             was edited, and a client must not silently accept either"
        );
    }

    #[test]
    fn a_chain_from_another_genesis_is_refused_without_being_weighed() {
        let real = Header::genesis(NetworkId::DEV);
        let honest = Branch {
            headers: vec![real, real.successor(1, EpochRoots::empty())],
            work: vec![1, 1],
        };

        // An attacker's network, internally consistent and far heavier.
        let mut forged_genesis = real;
        forged_genesis.roots.identity = Digest32::from_bytes([0xab; 32]);
        let forged = Branch {
            headers: vec![
                forged_genesis,
                forged_genesis.successor(1, EpochRoots::empty()),
            ],
            work: vec![u64::MAX / 2, u64::MAX / 2],
        };
        assert!(forged.weight() > honest.weight());
        assert!(
            network_rule().accepts(&honest, &forged).is_err(),
            "weight is never even consulted for a foreign genesis"
        );
    }
}
