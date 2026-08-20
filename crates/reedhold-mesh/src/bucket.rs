//! Grouping peers by shared key prefix.
//!
//! Knowing every node does not scale, and it is not needed. Keeping a handful
//! of contacts for each halving of the key space gives dense knowledge nearby,
//! sparse knowledge far away, and still routes in about `log(N)` hops.

use crate::ports::PeerId;
use crate::table::distance;

/// Peers kept per bucket. Kademlia's k.
pub const BUCKET_WIDTH: usize = 8;

/// Which bucket `peer` falls into, seen from `me`.
///
/// The index is the number of leading bits the two ids share, so bucket 0
/// holds the whole far half of the space and the highest buckets hold only
/// immediate neighbours.
#[must_use]
pub fn bucket_of(me: PeerId, peer: PeerId) -> u8 {
    let gap = distance(me, peer);
    let mut shared = 0_u32;
    for byte in gap {
        if byte == 0 {
            shared += 8;
            continue;
        }
        shared += byte.leading_zeros();
        break;
    }
    u8::try_from(shared.min(255)).unwrap_or(255)
}

#[cfg(test)]
mod tests {
    use super::bucket_of;
    use crate::ports::PeerId;
    use reedhold_core::Digest32;

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn buckets_group_by_shared_prefix() {
        let me = peer(0);
        let near = bucket_of(me, peer(1));
        let far = bucket_of(me, peer(128));
        assert!(near > far, "near {near} should out-rank far {far}");
        assert_eq!(bucket_of(me, me), 255, "self shares every bit");
    }

    #[test]
    fn a_rooted_table_keeps_only_k_per_bucket() {
        let me = peer(0);
        let mut table = crate::table::PeerTable::rooted(me);
        // 16..=31 share three leading zero bits, so sixteen peers compete for
        // one bucket that only holds eight.
        for byte in 1_u8..=31 {
            table.observe(peer(byte), u64::from(byte), None);
        }
        for probe in [peer(1), peer(4), peer(16), peer(31)] {
            assert!(
                table.bucket_len(probe) <= super::BUCKET_WIDTH,
                "bucket of {probe:?} held {}",
                table.bucket_len(probe)
            );
        }
        assert!(
            table.len() < 31,
            "a real node does not keep everyone, kept {}",
            table.len()
        );
    }
}
