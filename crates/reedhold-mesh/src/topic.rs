//! Topic-addressed delivery.
//!
//! A mailbox topic is a point in the same 256-bit space as a peer id, so one
//! routing table serves both: the fabric carries a payload toward the topic
//! exactly as it would toward a node, and whoever listens there collects it.
//! Nothing on the path reveals who the two ends are.

use crate::fabric::Fabric;
use crate::ports::PeerId;
use crate::route::Route;
use crate::walk::walk;
use reedhold_core::Result;

impl Fabric {
    /// Listen on `topic` and take anything already held for it.
    ///
    /// The backlog is copied, not consumed: a group topic has several readers,
    /// and whoever subscribed first must not swallow the others' mail. Each
    /// listener collects it once, on the subscription that first admits them.
    pub fn subscribe(&mut self, peer: PeerId, topic: PeerId) {
        self.nodes.entry(topic).or_default();
        if !self.topics.entry(topic).or_default().insert(peer) {
            return;
        }
        let waiting = self.peek_topic(topic);
        if let Some(node) = self.nodes.get_mut(&peer) {
            node.inbox.extend(waiting);
        }
    }

    /// Stop listening. Used when a mailbox epoch rolls over.
    pub fn unsubscribe(&mut self, peer: PeerId, topic: PeerId) {
        if let Some(listeners) = self.topics.get_mut(&topic) {
            listeners.remove(&peer);
        }
    }

    /// Deliver to whoever listens on `topic`, else park it near the topic.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Mesh`] when the sender is unknown.
    pub fn send_topic(&mut self, from: PeerId, topic: PeerId, payload: Vec<u8>) -> Result<Route> {
        let _ = self.node(from)?;
        self.nodes.entry(topic).or_default();
        self.table.observe(from, self.clock, None);
        let listeners = self.live_listeners(topic);
        if !listeners.is_empty() {
            for peer in listeners {
                if let Some(node) = self.nodes.get_mut(&peer) {
                    node.inbox.push(payload.clone());
                }
            }
            return Ok(Route::Direct);
        }
        if let Some(path) = walk(
            &self.table,
            |peer| self.is_live(peer),
            from,
            topic,
            self.clock,
        ) {
            let carrier = *path.last().unwrap_or(&from);
            self.hold_for(carrier, topic, payload)?;
            return Ok(Route::Hops(path));
        }
        if let Some(relay) = self.live_relay() {
            self.hold_for(relay, topic, payload)?;
            return Ok(Route::ViaRelay(relay));
        }
        // Nowhere closer to put it: keep it here, still filed under the topic.
        // A sender is a node like any other, so a listener that turns up later
        // finds it where it lies instead of it being stuck in an outbox.
        self.hold_for(from, topic, payload)?;
        Ok(Route::HeldLocal)
    }

    fn live_listeners(&self, topic: PeerId) -> Vec<PeerId> {
        self.topics
            .get(&topic)
            .into_iter()
            .flatten()
            .copied()
            .filter(|peer| self.is_live(*peer))
            .collect()
    }

    fn hold_for(&mut self, carrier: PeerId, topic: PeerId, payload: Vec<u8>) -> Result<()> {
        self.node_mut(carrier)?
            .hold
            .entry(topic)
            .or_default()
            .push(payload);
        Ok(())
    }

    fn peek_topic(&self, topic: PeerId) -> Vec<Vec<u8>> {
        self.nodes
            .values()
            .filter_map(|node| node.hold.get(&topic))
            .flatten()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::epoch::SyncEpoch;
    use crate::fabric::Fabric;
    use crate::plan::SyncPlan;
    use crate::ports::PeerId;
    use reedhold_core::{Digest32, NetworkId};

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn a_topic_holds_mail_until_its_listener_appears() {
        let pool: Vec<PeerId> = (1_u8..=8).map(peer).collect();
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 2 },
            &[1_u8; 32],
            &pool,
            None,
            2,
        );
        let mut mesh = Fabric::new(plan, &pool);
        let topic = peer(77);
        mesh.online(pool[0]).unwrap();
        mesh.online(pool[1]).unwrap();
        mesh.tick(30);

        // Nobody listens yet, so the payload waits in the network.
        mesh.send_topic(pool[0], topic, b"for whoever holds the key".to_vec())
            .unwrap();
        assert!(mesh.drain(pool[1]).unwrap().is_empty());

        mesh.subscribe(pool[1], topic);
        assert_eq!(
            mesh.drain(pool[1]).unwrap(),
            vec![b"for whoever holds the key".to_vec()]
        );
    }

    #[test]
    fn only_a_listener_of_that_topic_receives() {
        let pool: Vec<PeerId> = (1_u8..=8).map(peer).collect();
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 3 },
            &[2_u8; 32],
            &pool,
            None,
            2,
        );
        let mut mesh = Fabric::new(plan, &pool);
        for node in &pool {
            mesh.online(*node).unwrap();
        }
        mesh.subscribe(pool[1], peer(90));
        mesh.subscribe(pool[2], peer(91));
        mesh.send_topic(pool[0], peer(90), b"ours".to_vec())
            .unwrap();

        assert_eq!(mesh.drain(pool[1]).unwrap(), vec![b"ours".to_vec()]);
        assert!(
            mesh.drain(pool[2]).unwrap().is_empty(),
            "a neighbour on another topic learns nothing"
        );
    }
}
