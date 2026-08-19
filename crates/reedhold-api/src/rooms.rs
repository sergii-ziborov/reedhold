//! Public topic rooms. Slugs are local labels; packets carry identity hex only.

use crate::session::Session;
use reedhold_core::{ConversationId, Digest32, DomainTag, Error, IdentityId, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Suggested interest slugs. Clients may join any valid slug.
pub const TOPIC_CATALOG: &[&str] = &[
    "identity", "recovery", "mesh", "privacy", "storage", "ads", "work", "protocol",
];

/// One public message. Author is an identity hex, never a nick.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RoomPostView {
    /// Identity digest hex.
    pub from: String,
    /// UTF-8 body.
    pub text: String,
    /// Author device sequence at send.
    pub sequence: u64,
}

/// Public room snapshot.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RoomView {
    /// Conversation hex derived from the slug.
    pub id: String,
    /// Host-local slug. Not present in the signed event.
    pub topic: String,
    /// Member identity hexes.
    pub members: Vec<String>,
    /// Recent posts.
    pub posts: Vec<RoomPostView>,
}

/// In-process public rooms plus this user's interests.
#[derive(Clone, Debug, Default)]
pub struct RoomBoard {
    rooms: BTreeMap<ConversationId, Room>,
    slugs: BTreeMap<String, ConversationId>,
    interests: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct Room {
    slug: String,
    members: BTreeSet<IdentityId>,
    posts: Vec<RoomPostView>,
}

impl RoomBoard {
    /// Join or create a public room by slug. Company is irrelevant.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the slug is malformed.
    pub fn join(&mut self, session: &Session, slug: &str) -> Result<RoomView> {
        let slug = normalize_topic(slug)?;
        let id = room_id(&slug);
        let me = session.account.identity();
        let room = self.rooms.entry(id).or_insert_with(|| Room {
            slug: slug.clone(),
            members: BTreeSet::new(),
            posts: Vec::new(),
        });
        room.members.insert(me);
        self.slugs.insert(slug, id);
        Ok(room_view(id, room))
    }

    /// Leave a room. The room remains for others.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the slug is unknown.
    pub fn leave(&mut self, session: &Session, slug: &str) -> Result<()> {
        let slug = normalize_topic(slug)?;
        let id = self
            .slugs
            .get(&slug)
            .copied()
            .ok_or(Error::Event("unknown public room"))?;
        let room = self
            .rooms
            .get_mut(&id)
            .ok_or(Error::Event("unknown public room"))?;
        room.members.remove(&session.account.identity());
        Ok(())
    }

    /// Sign a public post. The slug is not copied into the event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the sender is not a member.
    pub fn post(&mut self, session: &mut Session, slug: &str, text: &str) -> Result<RoomPostView> {
        let slug = normalize_topic(slug)?;
        let id = self
            .slugs
            .get(&slug)
            .copied()
            .ok_or(Error::Event("unknown public room"))?;
        let me = session.account.identity();
        if !self
            .rooms
            .get(&id)
            .is_some_and(|room| room.members.contains(&me))
        {
            return Err(Error::Event("not a public room member"));
        }
        let event = session.emit("post", text)?;
        let item = RoomPostView {
            from: session.peer_hex(),
            text: text.to_owned(),
            sequence: event.sequence,
        };
        if let Some(room) = self.rooms.get_mut(&id) {
            room.posts.push(item.clone());
        }
        Ok(item)
    }

    /// Rooms this session has joined, or that match interests.
    #[must_use]
    pub fn list(&self, session: &Session) -> Vec<RoomView> {
        let me = session.account.identity();
        self.rooms
            .iter()
            .filter(|(_, room)| room.members.contains(&me) || self.interests.contains(&room.slug))
            .map(|(id, room)| room_view(*id, room))
            .collect()
    }

    /// Replace interest slugs. Invalid entries are skipped.
    pub fn set_interests(&mut self, topics: &[String]) {
        self.interests = topics
            .iter()
            .filter_map(|topic| normalize_topic(topic).ok())
            .collect();
    }

    /// Current interest slugs.
    #[must_use]
    pub fn interests(&self) -> Vec<String> {
        self.interests.iter().cloned().collect()
    }

    /// Built-in catalog.
    #[must_use]
    pub fn catalog() -> Vec<String> {
        TOPIC_CATALOG
            .iter()
            .map(|topic| (*topic).to_owned())
            .collect()
    }
}

fn room_id(slug: &str) -> ConversationId {
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::PublicRoom.as_bytes());
    hasher.update(slug.as_bytes());
    ConversationId::from_digest(Digest32::from_bytes(hasher.finalize().into()))
}

fn room_view(id: ConversationId, room: &Room) -> RoomView {
    RoomView {
        id: id.to_hex(),
        topic: room.slug.clone(),
        members: room.members.iter().map(|id| id.to_hex()).collect(),
        posts: room.posts.clone(),
    }
}

fn normalize_topic(raw: &str) -> Result<String> {
    let trimmed = raw.trim().trim_start_matches('#').to_ascii_lowercase();
    if trimmed.len() < 2 || trimmed.len() > 32 {
        return Err(Error::Identity("topic must be 2 to 32 characters"));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(Error::Identity("topic may only use a-z, 0-9, underscore"));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::RoomBoard;
    use crate::session::Session;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn public_post_does_not_embed_the_topic_alias() {
        let mut board = RoomBoard::default();
        let mut alice = Session::create("pw", &secret(3)).unwrap().session;
        board.join(&alice, "Privacy").unwrap();
        board.post(&mut alice, "privacy", "hello room").unwrap();
        let event = alice.history().unwrap().pop().unwrap();
        assert!(!event.event_hex.to_ascii_lowercase().contains("privacy"));
        assert_eq!(board.list(&alice)[0].posts[0].from, alice.peer_hex());
        assert_ne!(board.list(&alice)[0].posts[0].from, "privacy");
    }
}
