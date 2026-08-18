//! Per-peer mesh state. No sockets; a host or fabric drives this.

use crate::ports::PeerId;
use std::collections::BTreeMap;

/// One participant in the in-process fabric.
#[derive(Clone, Debug, Default)]
pub struct NodeState {
    /// Reachable for direct delivery.
    pub online: bool,
    /// Datagrams already delivered to this peer.
    pub inbox: Vec<Vec<u8>>,
    /// Mail this peer holds for others because it is a relay this epoch.
    pub hold: BTreeMap<PeerId, Vec<Vec<u8>>>,
    /// Outbound waiting for a dest or relay to come back.
    pub pending: Vec<(PeerId, Vec<u8>)>,
}

impl NodeState {
    /// Take delivered payloads.
    pub fn drain_inbox(&mut self) -> Vec<Vec<u8>> {
        core::mem::take(&mut self.inbox)
    }

    /// Take store-and-forward mail for `dest`.
    pub fn take_hold(&mut self, dest: PeerId) -> Vec<Vec<u8>> {
        self.hold.remove(&dest).unwrap_or_default()
    }
}
