//! Small-group state. Shared epoch key; MLS is a later stage.

use reedhold_core::{ConversationId, Digest32, DomainTag, Error, IdentityId, Result};
use reedhold_event::{InviteBody, MessageEnvelope, open_message, seal_message};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Local book for one small group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Circle {
    /// Conversation id.
    pub id: ConversationId,
    /// Current owner.
    pub owner: IdentityId,
    /// Key epoch. Bumped on membership rotation later.
    pub epoch: u64,
    /// Display name.
    pub name: String,
    /// Current members, including the owner.
    pub members: BTreeSet<IdentityId>,
    key: [u8; 32],
}

impl Circle {
    /// New group. The owner is the first member. The key never leaves this process
    /// except inside a sealed invite.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Entropy`] when the OS RNG is unavailable.
    pub fn create(owner: IdentityId, name: &str) -> Result<Self> {
        let mut nonce = [0_u8; 32];
        getrandom::getrandom(&mut nonce).map_err(|_| Error::Entropy)?;
        let mut hasher = Sha256::new();
        hasher.update(DomainTag::TalkGroup.as_bytes());
        hasher.update(owner.as_digest().as_bytes());
        hasher.update(name.as_bytes());
        hasher.update(nonce);
        let id = ConversationId::from_digest(Digest32::from_bytes(hasher.finalize().into()));
        let mut key = [0_u8; 32];
        getrandom::getrandom(&mut key).map_err(|_| Error::Entropy)?;
        let mut members = BTreeSet::new();
        members.insert(owner);
        Ok(Self {
            id,
            owner,
            epoch: 1,
            name: name.to_owned(),
            members,
            key,
        })
    }

    /// Rebuild local state from a decrypted invite.
    #[must_use]
    pub fn from_invite(invite: &InviteBody) -> Self {
        Self {
            id: invite.group,
            owner: invite.owner,
            epoch: invite.epoch,
            name: invite.name.clone(),
            members: invite.members.iter().copied().collect(),
            key: invite.key,
        }
    }

    /// Add a member to the local roster before wrapping an invite.
    pub fn include(&mut self, member: IdentityId) {
        self.members.insert(member);
    }

    /// Seal plaintext under the current epoch key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] on AEAD failure.
    pub fn seal(&self, plaintext: &[u8]) -> Result<MessageEnvelope> {
        seal_message(&self.key, plaintext)
    }

    /// Open a group envelope.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the key is wrong.
    pub fn open(&self, envelope: &MessageEnvelope) -> Result<Vec<u8>> {
        open_message(&self.key, envelope)
    }

    /// Invite payload for `member` (already on the roster).
    #[must_use]
    pub fn invite_body(&self) -> InviteBody {
        InviteBody {
            group: self.id,
            owner: self.owner,
            epoch: self.epoch,
            key: self.key,
            name: self.name.clone(),
            members: self.members.iter().copied().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Circle;
    use reedhold_core::{Digest32, IdentityId};
    use reedhold_event::open_message;

    fn id(byte: u8) -> IdentityId {
        IdentityId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn outsider_cannot_open_group_text() {
        let owner = id(1);
        let guest = id(2);
        let mut circle = Circle::create(owner, "room").unwrap();
        circle.include(guest);
        let sealed = circle.seal(b"secret").unwrap();
        assert_eq!(circle.open(&sealed).unwrap(), b"secret");
        assert!(open_message(&[0_u8; 32], &sealed).is_err());
        let restored = Circle::from_invite(&circle.invite_body());
        assert_eq!(restored.open(&sealed).unwrap(), b"secret");
    }
}
