//! Hard resource caps for a consumer node.

/// Per-node resource contract. Values are not frozen; the cap itself is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeBudget {
    /// Durable storage in bytes.
    pub durable_storage: u64,
    /// Opportunistic cache in bytes.
    pub opportunistic_cache: u64,
    /// Concurrent active peers.
    pub active_peers: u16,
    /// Routing-table peers.
    pub routing_peers: u16,
}

impl NodeBudget {
    /// Default phone-light profile from the protocol sketch.
    pub const PHONE_LIGHT: Self = Self {
        durable_storage: 5 * 1024 * 1024 * 1024,
        opportunistic_cache: 512 * 1024 * 1024,
        active_peers: 32,
        routing_peers: 256,
    };

    /// Whether `used` still fits the durable cap.
    #[must_use]
    pub const fn allows_durable(self, used: u64) -> bool {
        used <= self.durable_storage
    }
}

#[cfg(test)]
mod tests {
    use super::NodeBudget;

    #[test]
    fn phone_cap_is_finite() {
        let budget = NodeBudget::PHONE_LIGHT;
        assert!(budget.allows_durable(1024));
        assert!(!budget.allows_durable(u64::MAX));
    }
}
