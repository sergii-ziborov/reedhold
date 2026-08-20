//! Greedy routing: carry a payload to a peer that is not your neighbour.
//!
//! Each step must land strictly closer to the target in XOR key space. That
//! single rule is what makes the walk terminate: distance is a non-negative
//! integer that only ever decreases, so a loop is impossible.

use crate::ports::PeerId;
use crate::table::{PeerTable, distance};
use std::collections::BTreeSet;

/// Ceiling on forwarding steps. A greedy walk in a small-world graph settles
/// in about log(N); this only bounds the work when no path exists.
pub const MAX_HOPS: usize = 12;

/// How many candidates to consider at each step.
const FANOUT: usize = 8;

/// Path from `from` toward `to`, or `None` when nothing gets closer.
pub(crate) fn walk(
    table: &PeerTable,
    is_live: impl Fn(PeerId) -> bool,
    from: PeerId,
    to: PeerId,
    now: u64,
) -> Option<Vec<PeerId>> {
    let mut path = Vec::new();
    let mut here = from;
    let mut seen = BTreeSet::new();
    seen.insert(from);
    for _ in 0..MAX_HOPS {
        // Running out of closer neighbours ends the walk, it does not void it:
        // the payload still waits at the nearest node we did reach.
        let Some(best) = table.hops_toward(to, now, FANOUT).into_iter().find(|hop| {
            !seen.contains(hop) && is_live(*hop) && distance(*hop, to) < distance(here, to)
        }) else {
            break;
        };
        seen.insert(best);
        path.push(best);
        here = best;
        if is_live(to) {
            break;
        }
    }
    if path.is_empty() { None } else { Some(path) }
}

#[cfg(test)]
mod tests {
    use crate::epoch::SyncEpoch;
    use crate::fabric::Fabric;
    use crate::plan::SyncPlan;
    use crate::ports::PeerId;
    use crate::route::Route;
    use reedhold_core::{Digest32, NetworkId};

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    fn fabric(pool: &[PeerId], epoch: u64) -> Fabric {
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: epoch },
            &[5_u8; 32],
            pool,
            None,
            2,
        );
        Fabric::new(plan, pool)
    }

    #[test]
    fn a_payload_walks_toward_a_peer_that_is_not_a_relay() {
        let pool: Vec<PeerId> = (1_u8..=32).map(peer).collect();
        let mut mesh = fabric(&pool, 7);
        for node in &pool {
            mesh.online(*node).unwrap();
        }
        mesh.tick(60);
        // peer(16) has near neighbours in XOR space: peer(17) is one bit away,
        // while the sender peer(1) is seventeen. A greedy walk must use them.
        let target = peer(16);
        mesh.offline(target).unwrap();
        mesh.tick(120);

        let route = mesh.send(pool[0], target, b"walk".to_vec()).unwrap();
        assert_eq!(route.as_str(), "hops", "{route:?}");
        assert!(route.hop_count() <= super::MAX_HOPS);

        mesh.online(target).unwrap();
        assert_eq!(mesh.drain(target).unwrap(), vec![b"walk".to_vec()]);
    }

    #[test]
    fn a_remote_peer_is_handed_back_to_the_host() {
        let pool: Vec<PeerId> = (1_u8..=6).map(peer).collect();
        let mut mesh = fabric(&pool, 8);
        mesh.online(pool[0]).unwrap();
        let far = peer(200);
        mesh.link(far, "http://127.0.0.1:4784".to_owned());
        let route = mesh.send(pool[0], far, b"x".to_vec()).unwrap();
        assert_eq!(
            route,
            Route::Remote("http://127.0.0.1:4784".to_owned(), far)
        );
    }
}
