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

/// In-process directory. Replace with a DHT later; the API stays.
#[derive(Clone, Debug, Default)]
pub struct AliasDirectory {
    by_nick: BTreeMap<String, AliasView>,
    by_identity: BTreeMap<String, String>,
}

impl AliasDirectory {
    /// Claim or refresh `nick` for this session. Does not emit an event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the nick is malformed or taken.
    pub fn claim(&mut self, session: &Session, nick: &str) -> Result<AliasView> {
        let nick = normalize_nick(nick)?;
        let identity = session.peer_hex();
        if let Some(existing) = self.by_nick.get(&nick) {
            if existing.identity != identity {
                return Err(Error::Identity("alias already taken"));
            }
        }
        if let Some(old) = self.by_identity.get(&identity).cloned() {
            self.by_nick.remove(&old);
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
        directory.claim(&alice, "@Alice_01").unwrap();
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
}
