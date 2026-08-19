//! Decrypt delivered talk packets into host views.

use crate::session::Session;
use reedhold_core::{Digest32, Error, IdentityId, NetworkId, Result};
use reedhold_event::{EventKind, SignedEvent, TalkBody, TalkPacket, open_message};
use reedhold_protocol::Circle;
use serde::Serialize;

/// Public snapshot of a small group. Never includes the epoch key.
#[derive(Clone, Debug, Serialize)]
pub struct CircleView {
    /// Conversation hex.
    pub id: String,
    /// Owner identity hex.
    pub owner: String,
    /// Key epoch.
    pub epoch: u64,
    /// Display name.
    pub name: String,
    /// Member identity hexes.
    pub members: Vec<String>,
    /// True when this session is the owner-admin.
    pub you_admin: bool,
}

/// One decrypted inbox item.
#[derive(Clone, Debug, Serialize)]
pub struct TalkView {
    /// Event kind name.
    pub kind: String,
    /// Conversation hex.
    pub conversation: String,
    /// Author identity hex.
    pub from: String,
    /// UTF-8 plaintext, or the group name for invites.
    pub text: String,
}

pub(crate) fn ingest_one(session: &mut Session, item: &str) -> Result<TalkView> {
    let packet = TalkPacket::decode(&reedhold_core::decode_hex(item)?)?;
    let event = SignedEvent::decode_verify(&packet.event, NetworkId::DEV, &packet.device_public)?;
    if event.author != packet.author {
        return Err(Error::Event("talk author mismatch"));
    }
    // Group traffic follows group membership, which the owner already gates.
    // Only conversations a stranger can start are subject to the policy.
    let direct = matches!(
        event.kind,
        EventKind::DirectMessage | EventKind::GroupInvite
    );
    if direct && !session.accepts_from(packet.author) {
        return Err(Error::Event("sender is refused by this account"));
    }
    session.remember_pub(packet.author, packet.messaging_public);
    let body = TalkBody::decode(&packet.body)?;
    let (kind, text) = open_talk(session, &event, &packet, &body)?;
    Ok(TalkView {
        kind,
        conversation: body.conversation.to_hex(),
        from: packet.author.to_hex(),
        text,
    })
}

fn open_talk(
    session: &mut Session,
    event: &SignedEvent,
    packet: &TalkPacket,
    body: &TalkBody,
) -> Result<(String, String)> {
    match event.kind {
        EventKind::DirectMessage => {
            let key = session.account.messaging().agree(
                &packet.messaging_public,
                session.account.identity(),
                packet.author,
            )?;
            let plain = open_message(&key, &body.envelope)?;
            Ok((event.kind.as_str().to_owned(), utf8(&plain)?))
        }
        EventKind::GroupInvite => {
            let key = session.account.messaging().agree(
                &packet.messaging_public,
                session.account.identity(),
                packet.author,
            )?;
            let invite = reedhold_event::InviteBody::decode(&open_message(&key, &body.envelope)?)?;
            session.remember_circle(Circle::from_invite(&invite));
            Ok((event.kind.as_str().to_owned(), invite.name))
        }
        EventKind::GroupMessage => {
            let plain = session.circle(body.conversation)?.open(&body.envelope)?;
            Ok((event.kind.as_str().to_owned(), utf8(&plain)?))
        }
        EventKind::GroupLeave => {
            let plain = session.circle(body.conversation)?.open(&body.envelope)?;
            let raw: [u8; 32] = plain
                .as_slice()
                .try_into()
                .map_err(|_| Error::Event("leave payload has the wrong length"))?;
            let removed = IdentityId::from_digest(Digest32::from_bytes(raw));
            if removed == session.account.identity() {
                session.forget_circle(body.conversation);
            }
            Ok((event.kind.as_str().to_owned(), removed.to_hex()))
        }
        _ => Err(Error::Event("unsupported talk kind")),
    }
}

fn utf8(bytes: &[u8]) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| Error::Event("talk payload is not utf-8"))
}

pub(crate) fn circle_view(circle: &Circle) -> CircleView {
    circle_view_as(circle, circle.owner)
}

pub(crate) fn circle_view_as(circle: &Circle, me: IdentityId) -> CircleView {
    CircleView {
        id: circle.id.to_hex(),
        owner: circle.owner.to_hex(),
        epoch: circle.epoch,
        name: circle.name.clone(),
        members: circle.members.iter().map(|id| id.to_hex()).collect(),
        you_admin: circle.owner == me,
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Session;
    use crate::sync::sync_plan;
    use crate::talk::TalkNet;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    fn handset(byte: u8) -> Session {
        Session::create("pw", &secret(byte)).unwrap().session
    }

    #[test]
    fn dm_survives_relay_and_blocks() {
        let mut alice = handset(1);
        let mut bob = handset(2);
        let extras: Vec<String> = (10_u8..=16).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex()];
        candidates.extend(extras);
        let plan = sync_plan(4, &secret(0), &candidates, Some(&secret(99)), Some(2)).unwrap();
        let mut talk =
            TalkNet::open(4, &secret(0), &candidates, Some(&secret(99)), Some(2)).unwrap();
        talk.block(&secret(99)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&plan.relays[0]).unwrap();
        let route = talk
            .dm(
                &mut alice,
                &bob.peer_hex(),
                &bob.view().messaging_public,
                "hi",
            )
            .unwrap();
        assert_eq!(route.path, "relay");
        talk.online(&bob.peer_hex()).unwrap();
        let inbox = talk.inbox(&mut bob).unwrap();
        assert_eq!(inbox[0].text, "hi");
        assert_eq!(inbox[0].kind, "direct_message");
    }

    #[test]
    fn the_author_can_reread_what_they_sent() {
        let mut alice = handset(40);
        let bob = handset(41);
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
    fn a_newcomer_does_not_erase_mail_in_flight() {
        let mut alice = handset(42);
        let mut bob = handset(43);
        let late = handset(44);
        let extras: Vec<String> = (60_u8..=66).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(10, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.dm(
            &mut alice,
            &bob.peer_hex(),
            &bob.view().messaging_public,
            "held for bob",
        )
        .unwrap();
        talk.admit(&late.peer_hex()).unwrap();
        talk.online(&late.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();
        assert_eq!(talk.inbox(&mut bob).unwrap()[0].text, "held for bob");
    }

    #[test]
    fn small_group_invite_then_message() {
        let mut alice = handset(3);
        let mut bob = handset(4);
        let mut carol = handset(5);
        let extras: Vec<String> = (20_u8..=26).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex(), carol.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(5, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();
        talk.online(&carol.peer_hex()).unwrap();
        let group = talk.create_circle(&mut alice, "room").unwrap();
        talk.invite(
            &mut alice,
            &group.id,
            &bob.peer_hex(),
            &bob.view().messaging_public,
        )
        .unwrap();
        talk.invite(
            &mut alice,
            &group.id,
            &carol.peer_hex(),
            &carol.view().messaging_public,
        )
        .unwrap();
        assert_eq!(talk.inbox(&mut bob).unwrap()[0].kind, "group_invite");
        assert_eq!(talk.inbox(&mut carol).unwrap()[0].kind, "group_invite");
        talk.send_circle(&mut alice, &group.id, "hello").unwrap();
        assert_eq!(talk.inbox(&mut bob).unwrap()[0].text, "hello");
        assert_eq!(talk.inbox(&mut carol).unwrap()[0].text, "hello");
    }

    #[test]
    fn removed_member_cannot_read_new_epoch() {
        let mut alice = handset(6);
        let mut bob = handset(7);
        let mut carol = handset(8);
        let extras: Vec<String> = (30_u8..=36).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex(), carol.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(6, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();
        talk.online(&carol.peer_hex()).unwrap();
        let group = talk.create_circle(&mut alice, "room").unwrap();
        talk.invite(
            &mut alice,
            &group.id,
            &bob.peer_hex(),
            &bob.view().messaging_public,
        )
        .unwrap();
        talk.invite(
            &mut alice,
            &group.id,
            &carol.peer_hex(),
            &carol.view().messaging_public,
        )
        .unwrap();
        talk.inbox(&mut bob).unwrap();
        talk.inbox(&mut carol).unwrap();
        talk.remove(&mut alice, &group.id, &carol.peer_hex())
            .unwrap();
        assert_eq!(talk.inbox(&mut carol).unwrap()[0].kind, "group_leave");
        talk.inbox(&mut bob).unwrap();
        talk.send_circle(&mut alice, &group.id, "after").unwrap();
        assert_eq!(talk.inbox(&mut bob).unwrap()[0].text, "after");
        assert!(talk.inbox(&mut carol).unwrap().is_empty());
    }
}
