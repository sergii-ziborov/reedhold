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
        let view = contact_view(id, &entry);
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
        self.contacts
            .iter()
            .map(|(id, entry)| contact_view(*id, entry))
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

fn contact_view(id: IdentityId, entry: &ContactEntry) -> ContactView {
    ContactView {
        identity: id.to_hex(),
        messaging_public: reedhold_core::encode_hex(&entry.messaging_public),
        petname: entry.petname.clone(),
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
