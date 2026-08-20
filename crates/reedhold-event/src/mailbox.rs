//! Rotating mailbox addressing.
//!
//! Routing by identity publishes the social graph: every relay on a path sees
//! "A is talking to B" even when the body is sealed. A mailbox topic is
//! derived from the secret the two sides already share, so only they can
//! compute it, and it changes every epoch. From outside, two epochs of the
//! same conversation look like two unrelated random ids.

use crate::envelope::{MessageEnvelope, open_message, seal_message};
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Digest32, DomainTag, Error, Result};
use sha2::{Digest, Sha256};

const SEALED_TAG: u8 = 0x36;

/// How long one mailbox address stays in use, in seconds.
pub const MAILBOX_EPOCH_SECS: u64 = 6 * 3600;

/// `H(tag || shared_secret || epoch)`.
///
/// The secret never leaves the two endpoints, so nobody else can derive the
/// address, link two epochs of it, or tell which pair it belongs to.
#[must_use]
pub fn mailbox_topic(shared_secret: &[u8; 32], epoch: u64) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::Mailbox.as_bytes());
    hasher.update(shared_secret);
    hasher.update(epoch.to_be_bytes());
    Digest32::from_bytes(hasher.finalize().into())
}

/// Address anyone who looked you up can write to.
///
/// A pairwise topic needs both halves of a shared secret, so it cannot carry a
/// first message: the recipient has no way to derive it yet. This one comes
/// from the recipient's published messaging key alone.
///
/// The cost is honest and bounded: an observer who already knows that key can
/// see that *someone* wrote to this person, and still cannot tell who or what.
#[must_use]
pub fn delivery_topic(messaging_public: &[u8; 32], epoch: u64) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::Mailbox.as_bytes());
    hasher.update(b"delivery");
    hasher.update(messaging_public);
    hasher.update(epoch.to_be_bytes());
    Digest32::from_bytes(hasher.finalize().into())
}

/// Which mailbox epoch a wall-clock second belongs to.
#[must_use]
pub const fn mailbox_epoch(now_secs: u64) -> u64 {
    now_secs / MAILBOX_EPOCH_SECS
}

/// What actually travels the mesh.
///
/// Everything identifying — author, device key, messaging key, the signed
/// event itself — lives inside `sealed`. A carrier sees an address it cannot
/// attribute and bytes it cannot read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedPacket {
    /// Rotating mailbox address.
    pub topic: Digest32,
    /// Epoch the topic was derived for.
    pub epoch: u64,
    /// `TalkPacket` bytes under the shared key.
    pub sealed: MessageEnvelope,
}

impl SealedPacket {
    /// Wrap packet bytes for `shared_secret` at `epoch`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when sealing fails.
    pub fn seal(shared_secret: &[u8; 32], epoch: u64, packet: &[u8]) -> Result<Self> {
        Ok(Self {
            topic: mailbox_topic(shared_secret, epoch),
            epoch,
            sealed: seal_message(shared_secret, packet)?,
        })
    }

    /// Recover packet bytes. Fails when the secret is not the right one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the topic does not match the secret or
    /// the ciphertext does not open.
    pub fn open(&self, shared_secret: &[u8; 32]) -> Result<Vec<u8>> {
        if mailbox_topic(shared_secret, self.epoch) != self.topic {
            return Err(Error::Event("mailbox topic does not match this secret"));
        }
        open_message(shared_secret, &self.sealed)
    }

    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the envelope cannot be written.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(SEALED_TAG);
        writer.write_digest32(self.topic.as_bytes());
        writer.write_u64(self.epoch);
        writer.write_bytes(&self.sealed.encode()?)?;
        Ok(writer.finish())
    }

    /// Decode a sealed packet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the tag is unknown.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != SEALED_TAG {
            return Err(Error::Event("unknown sealed packet tag"));
        }
        let topic = Digest32::from_bytes(reader.read_digest32()?);
        let epoch = reader.read_u64()?;
        let sealed = MessageEnvelope::decode(reader.read_bytes()?)?;
        reader.finish()?;
        Ok(Self {
            topic,
            epoch,
            sealed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SealedPacket, mailbox_epoch, mailbox_topic};

    #[test]
    fn epochs_of_one_conversation_look_unrelated() {
        let secret = [9_u8; 32];
        let first = mailbox_topic(&secret, 100);
        let second = mailbox_topic(&secret, 101);
        assert_ne!(first, second, "the address must rotate");
        // No shared prefix an observer could group on.
        assert_ne!(first.as_bytes()[..8], second.as_bytes()[..8]);
    }

    #[test]
    fn a_stranger_cannot_derive_or_open_the_mailbox() {
        let ours = [1_u8; 32];
        let theirs = [2_u8; 32];
        let epoch = mailbox_epoch(7 * 3600);
        assert_ne!(mailbox_topic(&ours, epoch), mailbox_topic(&theirs, epoch));

        let packet = SealedPacket::seal(&ours, epoch, b"author and event live here").unwrap();
        assert!(packet.open(&theirs).is_err(), "wrong secret must not open");
        assert_eq!(packet.open(&ours).unwrap(), b"author and event live here");
    }

    #[test]
    fn nothing_identifying_survives_encoding() {
        let secret = [4_u8; 32];
        let wire = SealedPacket::seal(&secret, 3, b"alice-identity-bytes")
            .unwrap()
            .encode()
            .unwrap();
        assert!(
            !wire
                .windows(b"alice-identity-bytes".len())
                .any(|window| window == b"alice-identity-bytes"),
            "the author must not be readable on the wire"
        );
        let back = SealedPacket::decode(&wire).unwrap();
        assert_eq!(back.open(&secret).unwrap(), b"alice-identity-bytes");
    }
}
