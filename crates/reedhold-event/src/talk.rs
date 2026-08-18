//! Sealed talk payloads. Mesh carries these as opaque bytes.

use crate::envelope::MessageEnvelope;
use reedhold_codec::{Reader, Writer};
use reedhold_core::{ConversationId, Digest32, DomainTag, Error, IdentityId, Result};
use sha2::{Digest, Sha256};

const TALK_TAG: u8 = 0x31;
const INVITE_TAG: u8 = 0x32;
const PACKET_TAG: u8 = 0x35;
const MAX_MEMBERS: usize = 64;

/// Deterministic DM id. Same pair always yields the same conversation.
#[must_use]
pub fn dm_conversation(left: IdentityId, right: IdentityId) -> ConversationId {
    let (lo, hi) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::TalkPair.as_bytes());
    hasher.update(lo.as_digest().as_bytes());
    hasher.update(hi.as_digest().as_bytes());
    ConversationId::from_digest(Digest32::from_bytes(hasher.finalize().into()))
}

/// Conversation id plus a sealed envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TalkBody {
    /// DM or group id.
    pub conversation: ConversationId,
    /// Ciphertext under the pairwise or group epoch key.
    pub envelope: MessageEnvelope,
}

impl TalkBody {
    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the envelope cannot be written.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(TALK_TAG);
        writer.write_digest32(self.conversation.as_digest().as_bytes());
        writer.write_bytes(&self.envelope.encode()?)?;
        Ok(writer.finish())
    }

    /// Decode a talk body.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the tag is unknown.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != TALK_TAG {
            return Err(Error::Event("unknown talk tag"));
        }
        let conversation =
            ConversationId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
        let envelope = MessageEnvelope::decode(reader.read_bytes()?)?;
        reader.finish()?;
        Ok(Self {
            conversation,
            envelope,
        })
    }
}

/// Group epoch key wrapped for one member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InviteBody {
    /// Group conversation.
    pub group: ConversationId,
    /// Current owner.
    pub owner: IdentityId,
    /// Key epoch.
    pub epoch: u64,
    /// Shared group read key.
    pub key: [u8; 32],
    /// Display name.
    pub name: String,
    /// Current roster including the invitee.
    pub members: Vec<IdentityId>,
}

impl InviteBody {
    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the roster is too large.
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.members.len() > MAX_MEMBERS {
            return Err(Error::Event("group roster exceeds small-group limit"));
        }
        let count = u16::try_from(self.members.len())
            .map_err(|_| Error::Event("group roster exceeds small-group limit"))?;
        let mut writer = Writer::new();
        writer.write_u8(INVITE_TAG);
        writer.write_digest32(self.group.as_digest().as_bytes());
        writer.write_digest32(self.owner.as_digest().as_bytes());
        writer.write_u64(self.epoch);
        writer.write_digest32(&self.key);
        writer.write_bytes(self.name.as_bytes())?;
        writer.write_u16(count);
        for member in &self.members {
            writer.write_digest32(member.as_digest().as_bytes());
        }
        Ok(writer.finish())
    }

    /// Decode an invite.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the buffer is not an invite.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != INVITE_TAG {
            return Err(Error::Event("unknown invite tag"));
        }
        let group = ConversationId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
        let owner = IdentityId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
        let epoch = reader.read_u64()?;
        let key = reader.read_digest32()?;
        let name = String::from_utf8(reader.read_bytes()?.to_vec())
            .map_err(|_| Error::Event("invite name is not utf-8"))?;
        let count = usize::from(reader.read_u16()?);
        if count > MAX_MEMBERS {
            return Err(Error::Event("group roster exceeds small-group limit"));
        }
        let mut members = Vec::with_capacity(count);
        for _ in 0..count {
            members.push(IdentityId::from_digest(Digest32::from_bytes(
                reader.read_digest32()?,
            )));
        }
        reader.finish()?;
        Ok(Self {
            group,
            owner,
            epoch,
            key,
            name,
            members,
        })
    }
}

/// Signed sealed talk object plus the keys needed to verify and agree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TalkPacket {
    /// Author identity digest.
    pub author: IdentityId,
    /// Author X25519 public key.
    pub messaging_public: [u8; 32],
    /// Author device verifying key.
    pub device_public: [u8; 32],
    /// Canonical signed event.
    pub event: Vec<u8>,
    /// `TalkBody` bytes.
    pub body: Vec<u8>,
}

impl TalkPacket {
    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a field cannot be written.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(PACKET_TAG);
        writer.write_digest32(self.author.as_digest().as_bytes());
        writer.write_digest32(&self.messaging_public);
        writer.write_digest32(&self.device_public);
        writer.write_bytes(&self.event)?;
        writer.write_bytes(&self.body)?;
        Ok(writer.finish())
    }

    /// Decode a talk packet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the tag is unknown.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != PACKET_TAG {
            return Err(Error::Event("unknown talk packet tag"));
        }
        let author = IdentityId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
        let messaging_public = reader.read_digest32()?;
        let device_public = reader.read_digest32()?;
        let event = reader.read_bytes()?.to_vec();
        let body = reader.read_bytes()?.to_vec();
        reader.finish()?;
        Ok(Self {
            author,
            messaging_public,
            device_public,
            event,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{InviteBody, TalkBody, TalkPacket, dm_conversation};
    use crate::envelope::seal_message;
    use reedhold_core::{ConversationId, Digest32, IdentityId};

    #[test]
    fn dm_id_is_order_independent() {
        let a = IdentityId::from_digest(Digest32::from_bytes([1; 32]));
        let b = IdentityId::from_digest(Digest32::from_bytes([2; 32]));
        assert_eq!(dm_conversation(a, b), dm_conversation(b, a));
    }

    #[test]
    fn packet_round_trips() {
        let key = [3_u8; 32];
        let body = TalkBody {
            conversation: ConversationId::from_digest(Digest32::from_bytes([4; 32])),
            envelope: seal_message(&key, b"hi").unwrap(),
        };
        let packet = TalkPacket {
            author: IdentityId::from_digest(Digest32::from_bytes([5; 32])),
            messaging_public: [6; 32],
            device_public: [7; 32],
            event: b"evt".to_vec(),
            body: body.encode().unwrap(),
        };
        assert_eq!(
            TalkPacket::decode(&packet.encode().unwrap()).unwrap(),
            packet
        );
        let invite = InviteBody {
            group: ConversationId::from_digest(Digest32::from_bytes([4; 32])),
            owner: packet.author,
            epoch: 1,
            key,
            name: "room".into(),
            members: vec![packet.author],
        };
        assert_eq!(
            InviteBody::decode(&invite.encode().unwrap()).unwrap(),
            invite
        );
    }
}
