//! Sending and collecting under rotating mailbox addresses.
//!
//! Routing by identity puts the social graph on the wire. Here the address is
//! derived from the secret the two ends already share, so a carrier sees a
//! topic it cannot attribute, and the author travels inside the ciphertext.

use crate::mesh::RouteView;
use crate::session::Session;
use crate::talk::TalkNet;
use reedhold_core::{Digest32, Result, decode_hex, encode_hex};
use reedhold_event::{EventKind, SealedPacket, TalkBody, TalkPacket, mailbox_epoch, mailbox_topic};
use std::collections::BTreeMap;

impl TalkNet {
    /// Fabric time. Mailbox epochs are measured against it.
    #[must_use]
    pub fn now(&self) -> u64 {
        self.mesh.now()
    }

    /// Advance fabric time.
    pub fn tick(&mut self, now: u64) {
        self.mesh.tick(now);
    }

    /// Listen on every mailbox this session can compute.
    ///
    /// The previous epoch is watched too, so a message sent moments before a
    /// rollover is not stranded on an address nobody reads any more.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when a contact key cannot be agreed.
    pub fn listen(&mut self, session: &Session) -> Result<usize> {
        let me = session.peer_hex();
        let mut count = 0;
        for topic in self.mailbox_keys(session)?.keys() {
            self.mesh.subscribe(&me, *topic)?;
            count += 1;
        }
        Ok(count)
    }

    /// Seal a packet and post it to the conversation's current mailbox.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when sealing or routing fails.
    pub(crate) fn dispatch_sealed(
        &mut self,
        from: &mut Session,
        secret: &[u8; 32],
        kind: EventKind,
        body: &TalkBody,
    ) -> Result<RouteView> {
        let encoded = body.encode()?;
        let event = from.account.emit(kind, &encoded)?;
        from.push_log(&event, &encoded)?;
        let packet = TalkPacket {
            author: from.account.identity(),
            messaging_public: from.account.messaging().public_bytes(),
            device_public: from.account.device_public(),
            event: event.encode()?,
            body: encoded,
        };
        let sealed = SealedPacket::seal(secret, mailbox_epoch(self.mesh.now()), &packet.encode()?)?;
        let topic = sealed.topic;
        self.mesh
            .send_topic(&from.peer_hex(), topic, &encode_hex(&sealed.encode()?))
    }

    /// Every address this session reads, with the secret that opens it.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when a contact key cannot be agreed.
    pub(crate) fn mailbox_keys(&self, session: &Session) -> Result<BTreeMap<Digest32, [u8; 32]>> {
        let epoch = mailbox_epoch(self.mesh.now());
        let mut keyed = BTreeMap::new();
        for secret in session.mailbox_secrets()? {
            for slot in [epoch, epoch.saturating_sub(1)] {
                keyed.insert(mailbox_topic(&secret, slot), secret);
            }
        }
        Ok(keyed)
    }
}

/// Peel the mailbox envelope, or pass an identity-addressed packet through.
pub(crate) fn unwrap_item(item: &str, keyed: &BTreeMap<Digest32, [u8; 32]>) -> Option<String> {
    let raw = decode_hex(item).ok()?;
    let Ok(sealed) = SealedPacket::decode(&raw) else {
        return Some(item.to_owned());
    };
    let secret = keyed.get(&sealed.topic)?;
    Some(encode_hex(&sealed.open(secret).ok()?))
}

#[cfg(test)]
mod tests {
    use crate::session::Session;
    use crate::talk::TalkNet;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn a_dm_reaches_its_mailbox_and_no_identity_is_on_the_wire() {
        let mut alice = Session::create("pw", &secret(80)).unwrap().session;
        let mut bob = Session::create("pw", &secret(81)).unwrap().session;
        let extras: Vec<String> = (90_u8..=96).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(11, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.tick(9_000);
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();

        alice
            .add_contact(&bob.peer_hex(), &bob.view().messaging_public, "bob")
            .unwrap();
        bob.add_contact(&alice.peer_hex(), &alice.view().messaging_public, "alice")
            .unwrap();
        talk.listen(&bob).unwrap();

        let route = talk
            .dm(
                &mut alice,
                &bob.peer_hex(),
                &bob.view().messaging_public,
                "under a rotating address",
            )
            .unwrap();
        assert!(!route.path.is_empty());

        let inbox = talk.inbox(&mut bob).unwrap();
        assert_eq!(inbox.len(), 1, "the mailbox delivered");
        assert_eq!(inbox[0].text, "under a rotating address");
    }

    #[test]
    fn the_author_can_reread_what_they_sent() {
        let mut alice = Session::create("pw", &secret(40)).unwrap().session;
        let bob = Session::create("pw", &secret(41)).unwrap().session;
        let extras: Vec<String> = (50_u8..=56).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(9, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();

        let conversation =
            crate::talk::dm_conversation_hex(&alice.peer_hex(), &bob.peer_hex()).unwrap();
        talk.dm(
            &mut alice,
            &bob.peer_hex(),
            &bob.view().messaging_public,
            "mine",
        )
        .unwrap();
        assert_eq!(alice.thread(&conversation)[0].text, "mine");

        let solo = talk.create_circle(&mut alice, "solo").unwrap();
        talk.send_circle(&mut alice, &solo.id, "alone").unwrap();
        assert_eq!(alice.thread(&solo.id)[0].text, "alone");
    }

    #[test]
    fn a_stranger_listening_elsewhere_collects_nothing() {
        let mut alice = Session::create("pw", &secret(82)).unwrap().session;
        let mut bob = Session::create("pw", &secret(83)).unwrap().session;
        let mut eve = Session::create("pw", &secret(84)).unwrap().session;
        let extras: Vec<String> = (100_u8..=106).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex(), eve.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(12, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.tick(9_000);
        for who in [&alice, &bob, &eve] {
            talk.online(&who.peer_hex()).unwrap();
        }
        alice
            .add_contact(&bob.peer_hex(), &bob.view().messaging_public, "bob")
            .unwrap();
        bob.add_contact(&alice.peer_hex(), &alice.view().messaging_public, "alice")
            .unwrap();
        talk.listen(&bob).unwrap();
        talk.listen(&eve).unwrap();

        talk.dm(
            &mut alice,
            &bob.peer_hex(),
            &bob.view().messaging_public,
            "private",
        )
        .unwrap();

        assert_eq!(talk.inbox(&mut bob).unwrap().len(), 1);
        assert!(
            talk.inbox(&mut eve).unwrap().is_empty(),
            "eve cannot compute the address and receives nothing"
        );
    }
}
