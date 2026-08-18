//! Host API over the in-process mesh fabric.

use reedhold_core::{NetworkId, Result};
use reedhold_mesh::{DEFAULT_RELAY_COUNT, Fabric, PeerId, SyncEpoch, SyncPlan};
use serde::Serialize;

/// Result of one send attempt.
#[derive(Clone, Debug, Serialize)]
pub struct RouteView {
    /// `direct`, `relay`, or `held`.
    pub path: String,
    /// Relay peer hex when `path` is `relay`.
    pub hop: Option<String>,
}

/// In-process mesh. iOS/Android later inject a real link; this API stays.
pub struct MeshSession {
    fabric: Fabric,
}

impl MeshSession {
    /// Build a fabric from today's lottery. Peers start offline.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a hex id is invalid.
    pub fn open(
        epoch: u64,
        prior_commit_hex: &str,
        candidate_hexes: &[String],
        company_hex: Option<&str>,
        relay_count: Option<u16>,
    ) -> Result<Self> {
        let prior = reedhold_core::decode32(prior_commit_hex)?;
        let mut candidates = Vec::with_capacity(candidate_hexes.len());
        for hex in candidate_hexes {
            candidates.push(PeerId::from_hex(hex)?);
        }
        let company = match company_hex {
            Some(hex) if !hex.is_empty() => Some(PeerId::from_hex(hex)?),
            _ => None,
        };
        let limit =
            usize::from(relay_count.unwrap_or(u16::try_from(DEFAULT_RELAY_COUNT).unwrap_or(8)));
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: epoch },
            &prior,
            &candidates,
            company,
            limit,
        );
        Ok(Self {
            fabric: Fabric::new(plan, &candidates),
        })
    }

    /// Bring a peer online and deliver waiting mail.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is not in the fabric.
    pub fn online(&mut self, peer_hex: &str) -> Result<()> {
        self.fabric.online(PeerId::from_hex(peer_hex)?)
    }

    /// Take a peer offline.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is not in the fabric.
    pub fn offline(&mut self, peer_hex: &str) -> Result<()> {
        self.fabric.offline(PeerId::from_hex(peer_hex)?)
    }

    /// Block a host. The fabric keeps running.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex id is invalid.
    pub fn block(&mut self, peer_hex: &str) -> Result<()> {
        self.fabric.block(PeerId::from_hex(peer_hex)?);
        Ok(())
    }

    /// Send an opaque payload.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when a peer is unknown.
    pub fn send(&mut self, from_hex: &str, to_hex: &str, payload: &str) -> Result<RouteView> {
        let route = self.fabric.send(
            PeerId::from_hex(from_hex)?,
            PeerId::from_hex(to_hex)?,
            payload.as_bytes().to_vec(),
        )?;
        Ok(RouteView {
            path: route.as_str().to_owned(),
            hop: match route {
                reedhold_mesh::Route::ViaRelay(peer) => Some(peer.to_hex()),
                _ => None,
            },
        })
    }

    /// Drain delivered payloads as UTF-8 when possible, else hex.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn drain(&mut self, peer_hex: &str) -> Result<Vec<String>> {
        let packets = self.fabric.drain(PeerId::from_hex(peer_hex)?)?;
        Ok(packets
            .into_iter()
            .map(|bytes| match String::from_utf8(bytes.clone()) {
                Ok(text) => text,
                Err(_) => reedhold_core::encode_hex(&bytes),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::MeshSession;
    use crate::sync_plan;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn relay_delivers_after_recipient_returns() {
        let candidates: Vec<String> = (1_u8..=6).map(hex).collect();
        let plan = sync_plan(2, &hex(0), &candidates, None, Some(2)).unwrap();
        let mut mesh = MeshSession::open(2, &hex(0), &candidates, None, Some(2)).unwrap();
        let alice = candidates[0].clone();
        let bob = candidates
            .iter()
            .find(|peer| !plan.relays.contains(peer) && **peer != alice)
            .cloned()
            .unwrap();
        mesh.online(&alice).unwrap();
        mesh.online(&plan.relays[0]).unwrap();
        let route = mesh.send(&alice, &bob, "ping").unwrap();
        assert_eq!(route.path, "relay");
        mesh.online(&bob).unwrap();
        assert_eq!(mesh.drain(&bob).unwrap(), ["ping"]);
    }
}
