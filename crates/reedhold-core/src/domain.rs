//! Domain-separation labels used in hashes and key derivation.

/// Wire protocol name. Stable across implementations.
pub const PROTOCOL_NAME: &str = "reedhold";

/// Protocol version carried on events and manifests.
pub const PROTOCOL_VERSION: u16 = 1;

/// Domain-separated HKDF / hash labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainTag {
    /// `IdentityId = H(tag || identity-root public key)`.
    Identity,
    /// HKDF info for the identity signing root.
    IdentityRoot,
    /// HKDF info for device authorization material.
    DeviceRoot,
    /// HKDF info for messaging keys.
    MessagingRoot,
    /// HKDF info for recovery / vault material.
    RecoveryRoot,
    /// HKDF info for storage contracts.
    StorageRoot,
    /// HKDF info for local search indexes.
    SearchRoot,
    /// Content-addressed payload identifier.
    Content,
    /// Daily sync-epoch seed.
    SyncEpoch,
    /// Ranking of a peer as a transitional relay.
    RelayScore,
    /// Genesis advertising-root domain.
    AdvertisingRoot,
    /// Pairwise DM key from static X25519 agreement.
    TalkPair,
    /// Small-group conversation identifier.
    TalkGroup,
    /// Local seal for the persisted group book.
    CircleBook,
    /// Combined epoch state root.
    ChainState,
    /// Canonical chain header hash.
    ChainHeader,
    /// Merkle node / leaf for a checkpoint subtree.
    ChainMerkle,
}

impl DomainTag {
    /// Canonical UTF-8 label. Changing a label is a breaking protocol change.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "reedhold/identity/v1",
            Self::IdentityRoot => "reedhold/identity-root/v1",
            Self::DeviceRoot => "reedhold/device-root/v1",
            Self::MessagingRoot => "reedhold/messaging-root/v1",
            Self::RecoveryRoot => "reedhold/recovery-root/v1",
            Self::StorageRoot => "reedhold/storage-root/v1",
            Self::SearchRoot => "reedhold/search-root/v1",
            Self::Content => "reedhold/content/v1",
            Self::SyncEpoch => "reedhold/sync-epoch/v1",
            Self::RelayScore => "reedhold/relay-score/v1",
            Self::AdvertisingRoot => "reedhold/ads-root/v1",
            Self::TalkPair => "reedhold/talk-pair/v1",
            Self::TalkGroup => "reedhold/talk-group/v1",
            Self::CircleBook => "reedhold/circle-book/v1",
            Self::ChainState => "reedhold/chain-state/v1",
            Self::ChainHeader => "reedhold/chain-header/v1",
            Self::ChainMerkle => "reedhold/chain-merkle/v1",
        }
    }

    /// Label bytes for hashing and HKDF.
    #[must_use]
    pub const fn as_bytes(self) -> &'static [u8] {
        self.as_str().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::DomainTag;

    #[test]
    fn labels_are_unique() {
        let labels = [
            DomainTag::Identity,
            DomainTag::IdentityRoot,
            DomainTag::DeviceRoot,
            DomainTag::MessagingRoot,
            DomainTag::RecoveryRoot,
            DomainTag::StorageRoot,
            DomainTag::SearchRoot,
            DomainTag::Content,
            DomainTag::SyncEpoch,
            DomainTag::RelayScore,
            DomainTag::AdvertisingRoot,
            DomainTag::TalkPair,
            DomainTag::TalkGroup,
            DomainTag::CircleBook,
            DomainTag::ChainState,
            DomainTag::ChainHeader,
            DomainTag::ChainMerkle,
        ]
        .map(DomainTag::as_str);
        for (index, label) in labels.iter().enumerate() {
            assert!(labels[..index].iter().all(|other| other != label));
        }
    }
}
