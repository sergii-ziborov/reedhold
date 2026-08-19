//! Who may reach you, and what you no longer want to see.
//!
//! This is a session policy, not a transport rule. `MeshSession::block` cuts a
//! peer out of the fabric; these decide whether a delivered packet is allowed
//! to become a conversation.

use crate::session::Session;
use reedhold_core::{Error, IdentityId, Result};
use serde::Serialize;

/// Who is allowed to open a conversation with this account.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub enum MessagePolicy {
    /// Anyone may write. Strangers land in requests, not the main list.
    #[default]
    Everyone,
    /// Only people already in the address book get through.
    ContactsOnly,
    /// Nobody new. Existing contacts still reach you.
    Nobody,
}

impl MessagePolicy {
    /// Stable host-API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Everyone => "everyone",
            Self::ContactsOnly => "contacts",
            Self::Nobody => "nobody",
        }
    }

    /// Parse a host-API name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "everyone" => Some(Self::Everyone),
            "contacts" => Some(Self::ContactsOnly),
            "nobody" => Some(Self::Nobody),
            _ => None,
        }
    }
}

/// Current privacy state, as the host reports it.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct PrivacyView {
    /// Who may open a conversation.
    pub policy: String,
    /// Identity hexes you refuse.
    pub blocked: Vec<String>,
    /// Conversation hexes hidden from the main list.
    pub archived: Vec<String>,
}

impl Session {
    /// Snapshot of the privacy settings.
    #[must_use]
    pub fn privacy(&self) -> PrivacyView {
        PrivacyView {
            policy: self.policy.as_str().to_owned(),
            blocked: self.blocked.iter().map(|id| id.to_hex()).collect(),
            archived: self.archived.iter().cloned().collect(),
        }
    }

    /// Choose who may open a conversation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the name is not a known policy.
    pub fn set_policy(&mut self, name: &str) -> Result<PrivacyView> {
        self.policy =
            MessagePolicy::from_name(name).ok_or(Error::Identity("unknown message policy"))?;
        Ok(self.privacy())
    }

    /// Refuse this identity. Their delivered packets stop becoming messages.
    ///
    /// Blocking also drops them as a contact: keeping someone in the address
    /// book while refusing their mail is a contradiction the UI cannot show.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex is not an identity.
    pub fn block(&mut self, identity_hex: &str) -> Result<PrivacyView> {
        let id = IdentityId::from_hex(identity_hex)?;
        if id == self.account.identity() {
            return Err(Error::Identity("cannot block yourself"));
        }
        self.contacts.remove(&id);
        self.blocked.insert(id);
        Ok(self.privacy())
    }

    /// Let this identity through again. Does not restore the contact entry.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex is not an identity.
    pub fn unblock(&mut self, identity_hex: &str) -> Result<PrivacyView> {
        let id = IdentityId::from_hex(identity_hex)?;
        self.blocked.remove(&id);
        Ok(self.privacy())
    }

    /// True when this identity is refused.
    #[must_use]
    pub fn is_blocked(&self, id: IdentityId) -> bool {
        self.blocked.contains(&id)
    }

    /// Hide a conversation from the main list. History is kept.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex is malformed.
    pub fn archive(&mut self, conversation_hex: &str) -> Result<PrivacyView> {
        let key = normalize(conversation_hex)?;
        self.archived.insert(key);
        Ok(self.privacy())
    }

    /// Bring a conversation back into the main list.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex is malformed.
    pub fn unarchive(&mut self, conversation_hex: &str) -> Result<PrivacyView> {
        let key = normalize(conversation_hex)?;
        self.archived.remove(&key);
        Ok(self.privacy())
    }

    /// Whether a delivered packet from `author` is allowed to become a message.
    pub(crate) fn accepts_from(&self, author: IdentityId) -> bool {
        if self.blocked.contains(&author) {
            return false;
        }
        match self.policy {
            MessagePolicy::Everyone => true,
            MessagePolicy::ContactsOnly | MessagePolicy::Nobody => {
                self.contacts.contains_key(&author)
            }
        }
    }
}

fn normalize(conversation_hex: &str) -> Result<String> {
    let key = conversation_hex.trim().to_ascii_lowercase();
    if key.len() != 64 || !key.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(Error::Codec("conversation must be 32 hex bytes"));
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::MessagePolicy;
    use crate::session::Session;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn a_blocked_stranger_is_refused_and_leaves_the_address_book() {
        let mut me = Session::create("pw", &secret(70)).unwrap().session;
        let them = Session::create("pw", &secret(71)).unwrap().session;
        let hex = them.peer_hex();
        me.add_contact(&hex, &them.view().messaging_public, "them")
            .unwrap();
        assert_eq!(me.contacts().len(), 1);

        me.block(&hex).unwrap();
        assert!(me.contacts().is_empty(), "blocking removes the contact");
        assert!(!me.accepts_from(reedhold_core::IdentityId::from_hex(&hex).unwrap()));

        me.unblock(&hex).unwrap();
        assert!(me.accepts_from(reedhold_core::IdentityId::from_hex(&hex).unwrap()));
        assert!(me.block(&me.peer_hex()).is_err(), "you are not a stranger");
    }

    #[test]
    fn contacts_only_keeps_strangers_out_but_not_friends() {
        let mut me = Session::create("pw", &secret(72)).unwrap().session;
        let friend = Session::create("pw", &secret(73)).unwrap().session;
        let stranger = Session::create("pw", &secret(74)).unwrap().session;
        me.add_contact(&friend.peer_hex(), &friend.view().messaging_public, "f")
            .unwrap();

        me.set_policy(MessagePolicy::ContactsOnly.as_str()).unwrap();
        assert!(me.accepts_from(reedhold_core::IdentityId::from_hex(&friend.peer_hex()).unwrap()));
        assert!(
            !me.accepts_from(reedhold_core::IdentityId::from_hex(&stranger.peer_hex()).unwrap())
        );
        assert!(me.set_policy("whoever").is_err());
    }

    #[test]
    fn archiving_hides_a_conversation_without_losing_it() {
        let mut me = Session::create("pw", &secret(75)).unwrap().session;
        let key = "ab".repeat(32);
        me.archive(&key).unwrap();
        assert_eq!(me.privacy().archived, vec![key.clone()]);
        me.unarchive(&key).unwrap();
        assert!(me.privacy().archived.is_empty());
        assert!(me.archive("nothex").is_err());
    }
}
