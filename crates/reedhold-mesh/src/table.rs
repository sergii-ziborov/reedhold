//! Bounded peer table: who we know, how reliable they are, where they live.
//!
//! Discovery is not consensus. This table only answers "who should carry this
//! next", and it is capped so a phone never grows with the network.

use crate::ports::PeerId;
use std::collections::BTreeMap;

/// Ceiling on remembered peers. A consumer node stays O(1), not O(N).
pub const ROUTING_PEER_CAP: usize = 256;

/// Seconds of silence after which a peer stops counting as reachable.
pub const STALE_AFTER: u64 = 90;

/// What we have observed about one peer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PeerStat {
    /// First time we ever saw it.
    pub first_seen: u64,
    /// Last time it answered.
    pub last_seen: u64,
    /// Seconds it has been reachable across its life.
    pub live_secs: u64,
    /// Payloads it accepted for us.
    pub delivered: u32,
    /// Attempts that failed.
    pub failed: u32,
    /// Link address when the peer lives in another process.
    pub address: Option<String>,
}

impl PeerStat {
    /// Fraction of its life this peer was actually reachable, in milli.
    ///
    /// A peer that has been around a long time but is usually dark scores
    /// worse than a newcomer that is always up. Presence, not age.
    #[must_use]
    pub fn uptime_milli(&self, now: u64) -> u32 {
        let life = now.saturating_sub(self.first_seen).max(1);
        let ratio = self.live_secs.saturating_mul(1000) / life;
        u32::try_from(ratio.min(1000)).unwrap_or(1000)
    }

    /// Reachable if it answered recently enough.
    #[must_use]
    pub fn is_fresh(&self, now: u64) -> bool {
        now.saturating_sub(self.last_seen) <= STALE_AFTER
    }

    /// Uptime tempered by observed failures. Zero for a stale peer.
    #[must_use]
    pub fn score(&self, now: u64) -> u32 {
        if !self.is_fresh(now) {
            return 0;
        }
        let attempts = self.delivered.saturating_add(self.failed).max(1);
        let success = self.delivered.saturating_mul(1000) / attempts;
        let blended = self.uptime_milli(now).saturating_add(success) / 2;
        blended.max(1)
    }
}

/// XOR distance between two peers. Smaller means closer in key space.
#[must_use]
pub fn distance(left: PeerId, right: PeerId) -> [u8; 32] {
    let mut out = [0_u8; 32];
    let a = left.as_digest().as_bytes();
    let b = right.as_digest().as_bytes();
    for index in 0..32 {
        out[index] = a[index] ^ b[index];
    }
    out
}

/// Bounded routing table.
#[derive(Clone, Debug, Default)]
pub struct PeerTable {
    peers: BTreeMap<PeerId, PeerStat>,
    cap: usize,
}

impl PeerTable {
    /// Table with the protocol cap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            cap: ROUTING_PEER_CAP,
        }
    }

    /// Table with a custom cap. Used by simulations.
    #[must_use]
    pub fn with_cap(cap: usize) -> Self {
        Self {
            peers: BTreeMap::new(),
            cap: cap.max(1),
        }
    }

    /// Peers currently remembered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// True when nothing is known yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Read one peer.
    #[must_use]
    pub fn stat(&self, peer: PeerId) -> Option<&PeerStat> {
        self.peers.get(&peer)
    }

    /// Record that `peer` answered at `now`.
    ///
    /// Time reachable accumulates only across gaps short enough to count as
    /// continuous presence, so a peer cannot claim uptime for the hours it
    /// was gone.
    pub fn observe(&mut self, peer: PeerId, now: u64, address: Option<String>) {
        let cap = self.cap;
        let entry = self.peers.entry(peer).or_insert(PeerStat {
            first_seen: now,
            last_seen: now,
            ..PeerStat::default()
        });
        let gap = now.saturating_sub(entry.last_seen);
        if gap <= STALE_AFTER {
            entry.live_secs = entry.live_secs.saturating_add(gap);
        }
        entry.last_seen = now;
        if address.is_some() {
            entry.address = address;
        }
        if self.peers.len() > cap {
            self.evict(now);
        }
    }

    /// Note a successful hand-off.
    pub fn succeeded(&mut self, peer: PeerId) {
        if let Some(stat) = self.peers.get_mut(&peer) {
            stat.delivered = stat.delivered.saturating_add(1);
        }
    }

    /// Note a failed hand-off.
    pub fn failed(&mut self, peer: PeerId) {
        if let Some(stat) = self.peers.get_mut(&peer) {
            stat.failed = stat.failed.saturating_add(1);
        }
    }

    /// Forget the weakest peer once the table is over its cap.
    pub fn evict(&mut self, now: u64) {
        let worst = self
            .peers
            .iter()
            .min_by_key(|(peer, stat)| (stat.score(now), **peer))
            .map(|(peer, _)| *peer);
        if let Some(peer) = worst {
            self.peers.remove(&peer);
        }
    }

    /// Peers that could carry a payload toward `target`, best first.
    ///
    /// Ordering is XOR distance to the target, but only among peers that are
    /// actually answering: the closest node in key space is useless if it is
    /// dark. Ties break on score, then on id so the walk is deterministic.
    #[must_use]
    pub fn hops_toward(&self, target: PeerId, now: u64, limit: usize) -> Vec<PeerId> {
        let goal = distance(target, target);
        let mut live: Vec<(&PeerId, &PeerStat)> = self
            .peers
            .iter()
            .filter(|(peer, stat)| **peer != target && stat.score(now) > 0)
            .collect();
        live.sort_by_key(|(peer, stat)| {
            (distance(**peer, target), u32::MAX - stat.score(now), **peer)
        });
        let _ = goal;
        live.into_iter()
            .take(limit)
            .map(|(peer, _)| *peer)
            .collect()
    }

    /// Link address for a peer that lives in another process.
    #[must_use]
    pub fn address_of(&self, peer: PeerId) -> Option<String> {
        self.peers.get(&peer).and_then(|stat| stat.address.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{PeerTable, distance};
    use crate::ports::PeerId;
    use reedhold_core::Digest32;

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn distance_is_zero_to_self_and_symmetric() {
        assert_eq!(distance(peer(4), peer(4)), [0; 32]);
        assert_eq!(distance(peer(1), peer(2)), distance(peer(2), peer(1)));
    }

    #[test]
    fn a_dark_neighbour_never_wins_the_next_hop() {
        let mut table = PeerTable::new();
        // peer(2) is closest to the target in key space but went silent.
        table.observe(peer(2), 0, None);
        table.observe(peer(9), 0, None);
        table.observe(peer(9), 100, None);
        let hops = table.hops_toward(peer(3), 100, 4);
        assert_eq!(hops, vec![peer(9)], "only answering peers are offered");
    }

    #[test]
    fn presence_beats_age() {
        let mut table = PeerTable::new();
        table.observe(peer(1), 0, None);
        for tick in 1..=10 {
            table.observe(peer(1), tick * 10, None);
        }
        let steady = table.stat(peer(1)).unwrap().uptime_milli(100);
        table.observe(peer(2), 0, None);
        table.observe(peer(2), 100, None);
        let absent = table.stat(peer(2)).unwrap().uptime_milli(100);
        assert!(steady > absent, "{steady} should beat {absent}");
    }

    #[test]
    fn the_table_stays_bounded() {
        let mut table = PeerTable::with_cap(8);
        for byte in 1_u8..=40 {
            table.observe(peer(byte), u64::from(byte), None);
        }
        assert!(table.len() <= 8, "kept {}", table.len());
    }
}
