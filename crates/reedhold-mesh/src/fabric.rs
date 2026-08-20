//! In-process mesh fabric. UDP / libp2p / Freenet plug in later as links.

use crate::node::NodeState;
use crate::plan::SyncPlan;
use crate::ports::PeerId;
use crate::route::Route;
use crate::table::PeerTable;
use crate::walk::walk;
use reedhold_core::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Multi-node simulator and the first real routing implementation.
#[derive(Clone, Debug)]
pub struct Fabric {
    plan: SyncPlan,
    blocked: BTreeSet<PeerId>,
    nodes: BTreeMap<PeerId, NodeState>,
    table: PeerTable,
    clock: u64,
}

impl Fabric {
    /// All `candidates` start offline. Call [`Self::online`] to appear.
    #[must_use]
    pub fn new(plan: SyncPlan, candidates: &[PeerId]) -> Self {
        let mut nodes = BTreeMap::new();
        for peer in candidates {
            nodes.insert(*peer, NodeState::default());
        }
        if let Some(company) = plan.company {
            nodes.entry(company).or_default();
        }
        Self {
            plan,
            blocked: BTreeSet::new(),
            nodes,
            table: PeerTable::new(),
            clock: 0,
        }
    }

    /// Advance the fabric clock. Liveness and uptime are measured against it.
    pub fn tick(&mut self, now: u64) {
        self.clock = now.max(self.clock);
        let live: Vec<PeerId> = self
            .nodes
            .iter()
            .filter(|(peer, node)| node.online && !self.blocked.contains(peer))
            .map(|(peer, _)| *peer)
            .collect();
        for peer in live {
            self.table.observe(peer, self.clock, None);
        }
    }

    /// Read the routing table.
    #[must_use]
    pub const fn table(&self) -> &PeerTable {
        &self.table
    }

    /// Record where a peer can be reached when it is in another process.
    pub fn link(&mut self, peer: PeerId, address: String) {
        self.nodes.entry(peer).or_default();
        self.table.observe(peer, self.clock, Some(address));
    }

    /// Let a peer join a running fabric. Held mail is kept.
    ///
    /// A newcomer must never cost the network its undelivered mail, so this
    /// is the only way to grow the peer set: rebuilding the fabric drops
    /// every relay queue and every inbox.
    pub fn admit(&mut self, peer: PeerId) {
        self.nodes.entry(peer).or_default();
    }

    /// Mark a peer reachable.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn online(&mut self, peer: PeerId) -> Result<()> {
        self.node_mut(peer)?.online = true;
        self.flush_for(peer);
        Ok(())
    }

    /// Mark a peer unreachable. Mail on relays stays.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn offline(&mut self, peer: PeerId) -> Result<()> {
        self.node_mut(peer)?.online = false;
        Ok(())
    }

    /// Pretend a host is firewalled. Does not halt the fabric.
    pub fn block(&mut self, peer: PeerId) {
        self.blocked.insert(peer);
        if let Some(node) = self.nodes.get_mut(&peer) {
            node.online = false;
        }
    }

    /// Route `payload` from `from` to `to`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when either peer is unknown.
    pub fn send(&mut self, from: PeerId, to: PeerId, payload: Vec<u8>) -> Result<Route> {
        let _ = self.node(from)?;
        let _ = self.node(to)?;
        self.table.observe(from, self.clock, None);
        if self.is_live(to) {
            self.node_mut(to)?.inbox.push(payload);
            self.table.succeeded(to);
            return Ok(Route::Direct);
        }
        if let Some(address) = self.table.address_of(to) {
            return Ok(Route::Remote(address, to));
        }
        if let Some(path) = walk(&self.table, |peer| self.is_live(peer), from, to, self.clock) {
            let carrier = *path.last().unwrap_or(&to);
            self.node_mut(carrier)?
                .hold
                .entry(to)
                .or_default()
                .push(payload);
            self.table.succeeded(carrier);
            return Ok(Route::Hops(path));
        }
        if let Some(relay) = self.live_relay() {
            self.node_mut(relay)?
                .hold
                .entry(to)
                .or_default()
                .push(payload);
            return Ok(Route::ViaRelay(relay));
        }
        self.node_mut(from)?.pending.push((to, payload));
        Ok(Route::HeldLocal)
    }

    /// Take delivered payloads for `peer`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn drain(&mut self, peer: PeerId) -> Result<Vec<Vec<u8>>> {
        Ok(self.node_mut(peer)?.drain_inbox())
    }

    fn flush_for(&mut self, peer: PeerId) {
        let mut incoming = Vec::new();
        let holders: Vec<PeerId> = self.nodes.keys().copied().collect();
        for holder in holders {
            if holder == peer {
                continue;
            }
            if let Some(node) = self.nodes.get_mut(&holder) {
                incoming.extend(node.take_hold(peer));
            }
        }
        if let Some(node) = self.nodes.get_mut(&peer) {
            node.inbox.extend(incoming);
        }
        self.flush_pending();
    }

    fn flush_pending(&mut self) {
        let senders: Vec<PeerId> = self.nodes.keys().copied().collect();
        for sender in senders {
            let pending = self
                .nodes
                .get_mut(&sender)
                .map(|node| core::mem::take(&mut node.pending))
                .unwrap_or_default();
            for (dest, payload) in pending {
                let _ = self.send(sender, dest, payload);
            }
        }
    }

    fn live_relay(&self) -> Option<PeerId> {
        self.plan
            .relays
            .iter()
            .copied()
            .find(|relay| self.is_live(*relay))
    }

    fn is_live(&self, peer: PeerId) -> bool {
        !self.blocked.contains(&peer) && self.nodes.get(&peer).is_some_and(|node| node.online)
    }

    fn node(&self, peer: PeerId) -> Result<&NodeState> {
        self.nodes
            .get(&peer)
            .ok_or(Error::Mesh("unknown mesh peer"))
    }

    fn node_mut(&mut self, peer: PeerId) -> Result<&mut NodeState> {
        self.nodes
            .get_mut(&peer)
            .ok_or(Error::Mesh("unknown mesh peer"))
    }
}

#[cfg(test)]
mod tests {
    use super::Fabric;
    use crate::epoch::SyncEpoch;
    use crate::plan::SyncPlan;
    use crate::ports::PeerId;
    use crate::route::Route;
    use reedhold_core::{Digest32, NetworkId};

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    fn setup() -> (Fabric, PeerId, PeerId, PeerId, PeerId) {
        let pool: Vec<PeerId> = (1_u8..=8).map(peer).collect();
        let company = peer(99);
        let mut candidates = pool.clone();
        candidates.push(company);
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 3 },
            &[1_u8; 32],
            &candidates,
            Some(company),
            2,
        );
        let fabric = Fabric::new(plan, &candidates);
        (fabric, peer(1), peer(2), company, candidates[0])
    }

    #[test]
    fn store_and_forward_via_relay_then_deliver() {
        let (mut fabric, alice, bob, _, _) = setup();
        let relay = fabric.plan.relays[0];
        fabric.online(alice).unwrap();
        fabric.online(relay).unwrap();
        let route = fabric.send(alice, bob, b"hello".to_vec()).unwrap();
        assert_eq!(route, Route::ViaRelay(relay));
        fabric.online(bob).unwrap();
        assert_eq!(fabric.drain(bob).unwrap(), vec![b"hello".to_vec()]);
    }

    #[test]
    fn blocking_company_and_relays_still_allows_direct() {
        let (mut fabric, alice, bob, company, _) = setup();
        fabric.block(company);
        for relay in fabric.plan.relays.clone() {
            fabric.block(relay);
        }
        fabric.online(alice).unwrap();
        fabric.online(bob).unwrap();
        let route = fabric.send(alice, bob, b"x".to_vec()).unwrap();
        assert_eq!(route, Route::Direct);
        assert!(!fabric.plan.requires_company());
        assert!(!fabric.plan.blocking_is_fatal());
    }
}
