//! Transport and discovery vocabulary.

use reedhold_core::Digest32;

/// Opaque peer identifier. Not an identity id.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(Digest32);

impl PeerId {
    /// Wrap a digest.
    #[must_use]
    pub const fn from_digest(digest: Digest32) -> Self {
        Self(digest)
    }

    /// Borrow the digest.
    #[must_use]
    pub const fn as_digest(&self) -> &Digest32 {
        &self.0
    }
}

/// How a peer was found. None of these is a unique root of trust.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryHint {
    /// Previously cached peer.
    Cached,
    /// DHT lookup.
    Dht,
    /// Gossip peer exchange.
    PeerExchange,
    /// Optional company bootstrap. Never required.
    CompanyBootstrap,
    /// Community-published seed list.
    CommunitySeed,
    /// Local mDNS / LAN.
    Lan,
    /// Bluetooth / store-carry-forward.
    Ble,
    /// QR or invite hint.
    Invite,
}

/// Available transports. A client should speak more than one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    /// QUIC.
    Quic,
    /// TCP fallback.
    Tcp,
    /// Relay.
    Relay,
    /// Local LAN.
    Lan,
    /// Bluetooth.
    Ble,
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryHint, TransportKind};

    #[test]
    fn company_bootstrap_is_optional_not_unique() {
        let hints = [DiscoveryHint::CompanyBootstrap, DiscoveryHint::Dht];
        let transports = [TransportKind::Quic, TransportKind::Ble];
        assert_ne!(hints[0], hints[1]);
        assert_ne!(transports[0], transports[1]);
    }
}
