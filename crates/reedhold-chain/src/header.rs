//! Compact header. Fixed fields only: no DM, photo, or search index bytes.

use crate::hash::digest;
use crate::roots::EpochRoots;
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Digest32, DomainTag, Error, NetworkId, PROTOCOL_VERSION, Result};

const HEADER_TAG: u8 = 0x71;

/// One chain header. Light clients store a short window of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    /// Network isolation.
    pub network: NetworkId,
    /// Height. Genesis is 0.
    pub height: u64,
    /// Social epoch this header finalizes.
    pub epoch: u64,
    /// Hash of the previous header. Zeros at genesis.
    pub prev: Digest32,
    /// Subtree commitments.
    pub roots: EpochRoots,
}

impl Header {
    /// Genesis header. Empty roots, no predecessor.
    #[must_use]
    pub fn genesis(network: NetworkId) -> Self {
        Self {
            network,
            height: 0,
            epoch: 0,
            prev: Digest32::from_bytes([0; 32]),
            roots: EpochRoots::empty(),
        }
    }

    /// Next header after `self`.
    #[must_use]
    pub fn successor(&self, epoch: u64, roots: EpochRoots) -> Self {
        Self {
            network: self.network,
            height: self.height.saturating_add(1),
            epoch,
            prev: self.hash(),
            roots,
        }
    }

    /// Combined state root.
    #[must_use]
    pub fn state_root(&self) -> Digest32 {
        self.roots.state_root()
    }

    /// `H(chain-header || encode(header))`.
    #[must_use]
    pub fn hash(&self) -> Digest32 {
        digest(DomainTag::ChainHeader, &[self.encode().as_slice()])
    }

    /// Canonical encoding. Size does not depend on social payload.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.write_u8(HEADER_TAG);
        writer.write_u16(PROTOCOL_VERSION);
        writer.write_bytes(self.network.as_str().as_bytes()).ok();
        writer.write_u64(self.height);
        writer.write_u64(self.epoch);
        writer.write_digest32(self.prev.as_bytes());
        writer.write_digest32(self.roots.identity.as_bytes());
        writer.write_digest32(self.roots.groups.as_bytes());
        writer.write_digest32(self.roots.storage.as_bytes());
        writer.write_digest32(self.roots.reputation.as_bytes());
        writer.write_digest32(self.roots.ads.as_bytes());
        writer.write_digest32(self.roots.ledger.as_bytes());
        writer.write_digest32(self.state_root().as_bytes());
        writer.finish()
    }

    /// Decode and check the embedded state root.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] when the tag, network, or root is wrong.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != HEADER_TAG {
            return Err(Error::Chain("unknown header tag"));
        }
        if reader.read_u16()? != PROTOCOL_VERSION {
            return Err(Error::Chain("unsupported protocol version"));
        }
        if reader.read_bytes()? != NetworkId::DEV.as_str().as_bytes() {
            return Err(Error::Chain("network mismatch"));
        }
        let height = reader.read_u64()?;
        let epoch = reader.read_u64()?;
        let prev = Digest32::from_bytes(reader.read_digest32()?);
        let roots = EpochRoots {
            identity: Digest32::from_bytes(reader.read_digest32()?),
            groups: Digest32::from_bytes(reader.read_digest32()?),
            storage: Digest32::from_bytes(reader.read_digest32()?),
            reputation: Digest32::from_bytes(reader.read_digest32()?),
            ads: Digest32::from_bytes(reader.read_digest32()?),
            ledger: Digest32::from_bytes(reader.read_digest32()?),
        };
        let embedded = Digest32::from_bytes(reader.read_digest32()?);
        reader.finish()?;
        if embedded != roots.state_root() {
            return Err(Error::Chain("state root mismatch"));
        }
        Ok(Self {
            network: NetworkId::DEV,
            height,
            epoch,
            prev,
            roots,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::roots::EpochRoots;
    use reedhold_core::{Digest32, NetworkId};

    #[test]
    fn header_size_does_not_depend_on_payload() {
        let genesis = Header::genesis(NetworkId::DEV);
        let mut big = EpochRoots::empty();
        big.identity = Digest32::from_bytes([9; 32]);
        let next = genesis.successor(1, big);
        assert_eq!(genesis.encode().len(), next.encode().len());
        assert_eq!(Header::decode(&next.encode()).unwrap(), next);
        assert!(!next.encode().windows(5).any(|window| window == b"hello"));
    }
}
