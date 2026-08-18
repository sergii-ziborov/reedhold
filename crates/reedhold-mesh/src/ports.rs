//! Transport and discovery vocabulary.

use reedhold_core::Digest32;

/// Opaque peer identifier. Not an identity id.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// Hex form used by the host API.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.to_hex()
    }

    /// Parse 32 hex bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the string is not 32 bytes.
    pub fn from_hex(hex: &str) -> reedhold_core::Result<Self> {
        Ok(Self(Digest32::from_hex(hex)?))
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
    /// This epoch's randomly selected transitional relay.
    RotatingRelay,
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
