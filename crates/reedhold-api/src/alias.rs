//! Public nicknames. Untrusted lookup only — never written into events or keys.

use crate::session::Session;
use reedhold_core::{Error, Result};
use serde::Serialize;
use std::collections::BTreeMap;

/// Snapshot of a claimed alias. The nick is not an identity.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct AliasView {
    /// Public handle, without `@`. Not present on the wire.
    pub nick: String,
    /// Identity digest hex.
    pub identity: String,
    /// Static messaging public key hex.
    pub messaging_public: String,
}

/// How long a released nick stays unclaimable by anyone else.
///
/// Freeing a name the instant its owner renames lets a stranger take it and
/// receive everything meant for the person who left it behind.
pub const TOMBSTONE_SECS: u64 = 365 * 86_400;

/// In-process directory. Replace with a DHT later; the API stays.
#[derive(Clone, Debug, Default)]
pub struct AliasDirectory {
    by_nick: BTreeMap<String, AliasView>,
    by_identity: BTreeMap<String, String>,
    tombstones: BTreeMap<String, (String, u64)>,
}

impl AliasDirectory {
    /// Claim or refresh `nick` for this session. Does not emit an event.
    ///
    /// `now` is seconds since the epoch; it decides when released names come
    /// back into circulation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the nick is malformed, taken, or still
    /// held by its previous owner's tombstone.
    pub fn claim(&mut self, session: &Session, nick: &str, now: u64) -> Result<AliasView> {
        let nick = normalize_nick(nick)?;
        let identity = session.peer_hex();
        if let Some(existing) = self.by_nick.get(&nick) {
            if existing.identity != identity {
                return Err(Error::Identity("alias already taken"));
            }
        }
        if let Some((owner, until)) = self.tombstones.get(&nick) {
            if *owner != identity && now < *until {
                return Err(Error::Identity("alias is still held by its former owner"));
            }
        }
        self.tombstones.remove(&nick);
        if let Some(old) = self.by_identity.get(&identity).cloned() {
            self.by_nick.remove(&old);
            self.tombstones
                .insert(old, (identity.clone(), now.saturating_add(TOMBSTONE_SECS)));
        }
        let view = AliasView {
            nick: nick.clone(),
            identity: identity.clone(),
            messaging_public: session.view().messaging_public,
        };
        self.by_nick.insert(nick.clone(), view.clone());
        self.by_identity.insert(identity, nick);
        Ok(view)
    }

    /// Resolve a public nick to identity keys. Empty nick is not found.
    #[must_use]
    pub fn lookup(&self, nick: &str) -> Option<AliasView> {
        let Ok(nick) = normalize_nick(nick) else {
            return None;
        };
        self.by_nick.get(&nick).cloned()
    }

    /// Reverse display helper. Never used as a protocol id.
    #[must_use]
    pub fn nick_of(&self, identity_hex: &str) -> Option<String> {
        self.by_identity.get(identity_hex).cloned()
    }
}

/// Lower-case `[a-z0-9_]{3,32}`. Leading `@` is stripped.
///
/// # Errors
///
/// Returns [`Error::Identity`] when the nick is not an alias.
pub fn normalize_nick(raw: &str) -> Result<String> {
    let trimmed = raw.trim().trim_start_matches('@').to_ascii_lowercase();
    if trimmed.len() < 3 || trimmed.len() > 32 {
        return Err(Error::Identity("alias must be 3 to 32 characters"));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(Error::Identity("alias may only use a-z, 0-9, underscore"));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::AliasDirectory;
    use crate::session::Session;
    use crate::talk::TalkNet;

    fn secret(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn alias_is_not_written_into_talk_bytes() {
        let mut alice = Session::create("pw", &secret(1)).unwrap().session;
        let bob = Session::create("pw", &secret(2)).unwrap().session;
        let mut directory = AliasDirectory::default();
        directory.claim(&alice, "@Alice_01", 0).unwrap();
        let extras: Vec<String> = (10_u8..=16).map(secret).collect();
        let mut candidates = vec![alice.peer_hex(), bob.peer_hex()];
        candidates.extend(extras);
        let mut talk = TalkNet::open(1, &secret(0), &candidates, None, Some(2)).unwrap();
        talk.online(&alice.peer_hex()).unwrap();
        talk.online(&bob.peer_hex()).unwrap();
        let route = talk
            .dm(
                &mut alice,
                &bob.peer_hex(),
                &bob.view().messaging_public,
                "hi",
            )
            .unwrap();
        assert!(!route.path.is_empty());
        let event = alice.history().unwrap().pop().unwrap();
        assert!(!event.event_hex.to_ascii_lowercase().contains("alice_01"));
        assert!(
            !event
                .body_hex
                .to_ascii_lowercase()
                .contains("616c6963655f3031")
        );
        assert_eq!(
            directory.lookup("alice_01").unwrap().identity,
            alice.peer_hex()
        );
        assert!(directory.lookup("alice_01").unwrap().nick != alice.view().identity);
    }

    #[test]
    fn a_released_nick_is_not_up_for_grabs() {
        let alice = Session::create("pw", &secret(20)).unwrap().session;
        let mallory = Session::create("pw", &secret(21)).unwrap().session;
        let mut directory = AliasDirectory::default();
        directory.claim(&alice, "alice", 0).unwrap();
        directory.claim(&alice, "alice_two", 100).unwrap();

        assert!(
            directory.lookup("alice").is_none(),
            "the old nick is retired"
        );
        assert!(
            directory.claim(&mallory, "alice", 200).is_err(),
            "a stranger cannot inherit the name people knew her by"
        );
        assert!(
            directory.claim(&alice, "alice", 200).is_ok(),
            "the original owner may take it back at any time"
        );

        let mut later = AliasDirectory::default();
        later.claim(&alice, "alice", 0).unwrap();
        later.claim(&alice, "alice_two", 0).unwrap();
        assert!(
            later
                .claim(&mallory, "alice", super::TOMBSTONE_SECS + 1)
                .is_ok(),
            "names return to circulation once the hold expires"
        );
    }
}
