//! Sync plan for one epoch. Company host is never required.

use crate::epoch::{EpochSeed, SyncEpoch};
use crate::lottery::select_relays;
use crate::ports::PeerId;
use reedhold_core::NetworkId;

/// Default number of transitional relays drawn each day.
pub const DEFAULT_RELAY_COUNT: usize = 8;

/// How a host participates this epoch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostRole {
    /// Ordinary peer. May still talk P2P / LAN / BLE.
    Peer,
    /// Temporary store-and-forward helper for this epoch only.
    RotatingRelay,
    /// Optional company accelerator. Blocking it must do nothing.
    OptionalCompany,
}

/// Preferred sync set for one day. Not a trust root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncPlan {
    /// Epoch this roster belongs to.
    pub epoch: SyncEpoch,
    /// Today's randomly selected transitional hosts.
    pub relays: Vec<PeerId>,
    /// Company site, if anyone advertised one. Never required.
    pub company: Option<PeerId>,
}

impl SyncPlan {
    /// Draw today's relays from the live peer set.
    ///
    /// `company` is recorded as optional and stripped from the lottery so it
    /// cannot occupy a "required" slot.
    #[must_use]
    pub fn draw(
        network: NetworkId,
        epoch: SyncEpoch,
        prior_commit: &[u8; 32],
        candidates: &[PeerId],
        company: Option<PeerId>,
        relay_count: usize,
    ) -> Self {
        let seed = EpochSeed::derive(network, epoch, prior_commit);
        let pool: Vec<PeerId> = candidates
            .iter()
            .copied()
            .filter(|peer| company != Some(*peer))
            .collect();
        Self {
            epoch,
            relays: select_relays(&seed, &pool, relay_count),
            company,
        }
    }

    /// Role of `peer` in this plan.
    #[must_use]
    pub fn role(&self, peer: PeerId) -> HostRole {
        if self.company == Some(peer) {
            HostRole::OptionalCompany
        } else if self.relays.contains(&peer) {
            HostRole::RotatingRelay
        } else {
            HostRole::Peer
        }
    }

    /// Company bootstrap is never a protocol requirement.
    #[must_use]
    pub const fn requires_company(&self) -> bool {
        false
    }

    /// Blocking company and/or today's relays does not halt the mesh.
    ///
    /// Remaining peers still gossip, LAN/BLE still work, and tomorrow's
    /// lottery picks a different set.
    #[must_use]
    pub const fn blocking_is_fatal(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_RELAY_COUNT, SyncPlan};
    use crate::epoch::SyncEpoch;
    use crate::ports::PeerId;
    use reedhold_core::{Digest32, NetworkId};

    fn peer(byte: u8) -> PeerId {
        PeerId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    fn candidates() -> Vec<PeerId> {
        (1_u8..=32).map(peer).collect()
    }

    #[test]
    fn company_is_never_required_and_not_a_lottery_winner() {
        let company = peer(99);
        let mut pool = candidates();
        pool.push(company);
        let plan = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 4 },
            &[7_u8; 32],
            &pool,
            Some(company),
            DEFAULT_RELAY_COUNT,
        );
        assert!(!plan.requires_company());
        assert!(!plan.blocking_is_fatal());
        assert_eq!(plan.company, Some(company));
        assert!(!plan.relays.contains(&company));
        assert_eq!(plan.role(company), super::HostRole::OptionalCompany);
    }

    #[test]
    fn next_day_redraws_the_roster() {
        let pool = candidates();
        let today = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 20 },
            &[3_u8; 32],
            &pool,
            None,
            DEFAULT_RELAY_COUNT,
        );
        let tomorrow = SyncPlan::draw(
            NetworkId::DEV,
            SyncEpoch { index: 21 },
            &[4_u8; 32],
            &pool,
            None,
            DEFAULT_RELAY_COUNT,
        );
        assert_ne!(today.relays, tomorrow.relays);
        let mut blocked = today.relays.clone();
        blocked.extend(today.company);
        assert!(!today.blocking_is_fatal());
        assert!(tomorrow.relays.iter().any(|peer| !blocked.contains(peer)));
    }
}
