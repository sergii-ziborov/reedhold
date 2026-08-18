//! Mesh datagram. Not a social event; payload is opaque.

use crate::ports::PeerId;
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Error, Result};

const FRAME_TAG: u8 = 0x61;

/// How this datagram should be handled by the next hop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameKind {
    /// Deliver to `to` now.
    Direct,
    /// Store at the hop until `to` is reachable.
    Relay,
}

impl FrameKind {
    const fn as_u8(self) -> u8 {
        match self {
            Self::Direct => 1,
            Self::Relay => 2,
        }
    }

    const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Direct),
            2 => Some(Self::Relay),
            _ => None,
        }
    }
}

/// One mesh hop. Company is never implied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshFrame {
    /// Origin peer.
    pub from: PeerId,
    /// Final recipient.
    pub to: PeerId,
    /// Direct or store-and-forward.
    pub kind: FrameKind,
    /// Opaque payload (usually a sealed social object).
    pub payload: Vec<u8>,
}

impl MeshFrame {
    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the payload cannot be written.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(FRAME_TAG);
        writer.write_digest32(self.from.as_digest().as_bytes());
        writer.write_digest32(self.to.as_digest().as_bytes());
        writer.write_u8(self.kind.as_u8());
        writer.write_bytes(&self.payload)?;
        Ok(writer.finish())
    }

    /// Decode a canonical frame.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the tag or kind is unknown.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != FRAME_TAG {
            return Err(Error::Mesh("unknown frame tag"));
        }
        let from =
            PeerId::from_digest(reedhold_core::Digest32::from_bytes(reader.read_digest32()?));
        let to = PeerId::from_digest(reedhold_core::Digest32::from_bytes(reader.read_digest32()?));
        let kind =
            FrameKind::from_u8(reader.read_u8()?).ok_or(Error::Mesh("unknown frame kind"))?;
        let payload = reader.read_bytes()?.to_vec();
        reader.finish()?;
        Ok(Self {
            from,
            to,
            kind,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameKind, MeshFrame};
    use crate::ports::PeerId;
    use reedhold_core::Digest32;

    #[test]
    fn frame_round_trips() {
        let frame = MeshFrame {
            from: PeerId::from_digest(Digest32::from_bytes([1; 32])),
            to: PeerId::from_digest(Digest32::from_bytes([2; 32])),
            kind: FrameKind::Relay,
            payload: b"hi".to_vec(),
        };
        let encoded = frame.encode().unwrap();
        assert_eq!(MeshFrame::decode(&encoded).unwrap(), frame);
    }
}
