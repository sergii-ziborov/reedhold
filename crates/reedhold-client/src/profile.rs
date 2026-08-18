//! Consumer node profile.

use reedhold_mesh::TransportKind;
use reedhold_storage::NodeBudget;

/// How a client is allowed to participate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientProfile {
    /// Hard resource caps.
    pub budget: NodeBudget,
    /// Transports this profile will attempt.
    pub transports: &'static [TransportKind],
    /// Whether the node may take durable storage contracts.
    pub durable_seeder: bool,
}

impl ClientProfile {
    /// Phone / mobile profile.
    pub const PHONE: Self = Self {
        budget: NodeBudget::PHONE_LIGHT,
        transports: &[
            TransportKind::Quic,
            TransportKind::Tcp,
            TransportKind::Relay,
            TransportKind::Lan,
            TransportKind::Ble,
        ],
        durable_seeder: false,
    };

    /// Desktop profile. Still bounded, but may seed.
    pub const DESKTOP: Self = Self {
        budget: NodeBudget {
            durable_storage: 50 * 1024 * 1024 * 1024,
            opportunistic_cache: 2 * 1024 * 1024 * 1024,
            active_peers: 64,
            routing_peers: 512,
        },
        transports: &[TransportKind::Quic, TransportKind::Tcp, TransportKind::Lan],
        durable_seeder: true,
    };
}

#[cfg(test)]
mod tests {
    use super::ClientProfile;

    #[test]
    fn phone_is_not_a_seeder() {
        let phone = ClientProfile::PHONE;
        let desktop = ClientProfile::DESKTOP;
        assert!(!phone.durable_seeder);
        assert!(desktop.durable_seeder);
        assert!(phone.budget.durable_storage < desktop.budget.durable_storage);
    }
}
