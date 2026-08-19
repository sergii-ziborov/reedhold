//! Local address book. Petnames stay on this device.

use crate::inbox::{CircleView, circle_view_as};
use crate::session::Session;
use reedhold_core::{Error, IdentityId, Result, decode32};
use serde::Serialize;

/// One local contact. The petname is never sent.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ContactView {
    /// Identity digest hex.
    pub identity: String,
    /// Messaging public key hex.
    pub messaging_public: String,
    /// Local label. Empty if unset. Not an alias, not in crypto.
    pub petname: String,
    /// Deterministic DM conversation hex with this contact.
    ///
    /// A transcript is keyed by conversation, never by identity. Clients that
    /// guess the key silently show an empty chat.
    pub conversation: String,
}

/// Someone wrote who is not in the address book yet.
///
/// Without this a stranger's message lands in the transcript with no chat row
/// to show it in, and looks exactly like a message that never arrived.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RequestView {
    /// Sender identity hex.
    pub identity: String,
    /// Sender messaging public key hex, learned from the packet.
    pub messaging_public: String,
    /// Conversation hex the messages are filed under.
    pub conversation: String,
    /// How many messages are waiting.
    pub count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct ContactEntry {
    pub(crate) messaging_public: [u8; 32],
    pub(crate) petname: String,
}

impl Session {
    /// Remember someone by keys. `petname` is local-only.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a hex field is the wrong length.
    pub fn add_contact(
        &mut self,
        identity_hex: &str,
        messaging_public_hex: &str,
        petname: &str,
    ) -> Result<ContactView> {
        let id = IdentityId::from_hex(identity_hex)?;
        let public = decode32(messaging_public_hex)?;
        self.remember_pub(id, public);
        let entry = ContactEntry {
            messaging_public: public,
            petname: petname.trim().to_owned(),
        };
        let view = contact_view_with(self.account.identity(), id, &entry);
        self.contacts.insert(id, entry);
        Ok(view)
    }

    /// Drop a local contact. Identity is unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the contact is unknown.
    pub fn remove_contact(&mut self, identity_hex: &str) -> Result<()> {
        let id = IdentityId::from_hex(identity_hex)?;
        self.contacts
            .remove(&id)
            .ok_or(Error::Identity("unknown contact"))?;
        Ok(())
    }

    /// Local book, sorted by identity hex.
    #[must_use]
    pub fn contacts(&self) -> Vec<ContactView> {
        let me = self.account.identity();
        self.contacts
            .iter()
            .map(|(id, entry)| contact_view_with(me, *id, entry))
            .collect()
    }

    /// Conversations opened by people who are not contacts yet.
    ///
    /// Group traffic is excluded: a group already has its own chat row.
    #[must_use]
    pub fn requests(&self) -> Vec<RequestView> {
        let me = self.account.identity();
        let mut seen: std::collections::BTreeMap<IdentityId, (String, usize)> =
            std::collections::BTreeMap::new();
        for (conversation, items) in &self.threads {
            if reedhold_core::ConversationId::from_hex(conversation)
                .is_ok_and(|id| self.circles.contains_key(&id))
            {
                continue;
            }
            for item in items {
                let Ok(from) = IdentityId::from_hex(&item.from) else {
                    continue;
                };
                if from == me || self.contacts.contains_key(&from) {
                    continue;
                }
                let entry = seen.entry(from).or_insert((conversation.clone(), 0));
                entry.1 = entry.1.saturating_add(1);
            }
        }
        seen.into_iter()
            .map(|(id, (conversation, count))| RequestView {
                identity: id.to_hex(),
                messaging_public: self
                    .pubs
                    .get(&id)
                    .map(|key| reedhold_core::encode_hex(key))
                    .unwrap_or_default(),
                conversation,
                count,
            })
            .collect()
    }

    /// Groups this session already knows, with admin flag for the owner.
    #[must_use]
    pub fn circles(&self) -> Vec<CircleView> {
        let me = self.account.identity();
        self.circles
            .values()
            .map(|circle| circle_view_as(circle, me))
            .collect()
    }
}

fn contact_view_with(me: IdentityId, id: IdentityId, entry: &ContactEntry) -> ContactView {
    ContactView {
        identity: id.to_hex(),
        messaging_public: reedhold_core::encode_hex(&entry.messaging_public),
        petname: entry.petname.clone(),
        conversation: reedhold_event::dm_conversation(me, id).to_hex(),
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Session;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn petname_stays_local() {
        let mut alice = Session::create("pw", &secret(1)).unwrap().session;
        let bob = Session::create("pw", &secret(2)).unwrap().session;
        alice
            .add_contact(&bob.peer_hex(), &bob.view().messaging_public, "Bob")
            .unwrap();
        assert_eq!(alice.contacts()[0].petname, "Bob");
        assert_eq!(alice.contacts()[0].identity, bob.peer_hex());
        alice.remove_contact(&bob.peer_hex()).unwrap();
        assert!(alice.contacts().is_empty());
    }
}
