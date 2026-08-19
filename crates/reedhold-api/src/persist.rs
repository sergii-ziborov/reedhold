//! Directory save/load. The host picks the path (app sandbox, not our home).

use crate::session::Session;
use crate::view::EventView;
use reedhold_core::{NetworkId, Result};
use reedhold_event::{MessageEnvelope, SignedEvent, open_message, seal_message};
use reedhold_protocol::{Circle, restore_account};
use reedhold_store::LocalStore;

impl Session {
    /// Write the sealed manifest and signed event log to `dir`.
    ///
    /// # Errors
    ///
    /// Returns a codec error when the directory cannot be written.
    pub fn save(&self, dir: &str) -> Result<()> {
        let store = LocalStore::open(dir);
        store.save_manifest(self.account.manifest())?;
        store.save_events(&self.log)?;
        store.save_circles(&seal_book(self)?)
    }

    /// Restore from a wiped process using only the directory + password.
    ///
    /// # Errors
    ///
    /// Returns a recovery error when the password is wrong or events fail
    /// verification.
    pub fn load(dir: &str, password: &str, device_secret_hex: &str) -> Result<Self> {
        let store = LocalStore::open(dir);
        let manifest = store.load_manifest(NetworkId::DEV)?;
        let device = reedhold_core::decode32(device_secret_hex)?;
        let account = restore_account(&manifest, password.as_bytes(), &device)?;
        let raw = store.load_events()?;
        for event in &raw {
            SignedEvent::decode_verify(&event.encoded, NetworkId::DEV, &account.device_public())?;
        }
        let mut session = Self::from_parts(account, raw);
        if let Some(bytes) = store.load_circles()? {
            open_book(&mut session, &bytes)?;
        }
        Ok(session)
    }

    /// Verified events currently held in this session.
    ///
    /// # Errors
    ///
    /// Returns a codec error when a stored event cannot be parsed.
    pub fn history(&self) -> Result<Vec<EventView>> {
        let mut views = Vec::with_capacity(self.log.len());
        for event in &self.log {
            let signed = SignedEvent::decode_verify(
                &event.encoded,
                NetworkId::DEV,
                &self.account.device_public(),
            )?;
            views.push(EventView::from_event(&signed, &event.body)?);
        }
        Ok(views)
    }
}

fn seal_book(session: &Session) -> Result<Vec<u8>> {
    let circles: Vec<Circle> = session.circles.values().cloned().collect();
    let plain = Circle::encode_book(&circles)?;
    let key = session.account.messaging().book_key()?;
    seal_message(&key, &plain)?.encode()
}

fn open_book(session: &mut Session, bytes: &[u8]) -> Result<()> {
    let key = session.account.messaging().book_key()?;
    let plain = open_message(&key, &MessageEnvelope::decode(bytes)?)?;
    for circle in Circle::decode_book(&plain)? {
        session.remember_circle(circle);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::session::Session;

    #[test]
    fn wipe_process_reinstalls_from_directory() {
        let secret = "22".repeat(32);
        let dir = std::env::temp_dir().join(format!("reedhold-reinstall-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut created = Session::create("pw", &secret).unwrap();
        let identity = created.session.view().identity;
        created.session.emit("post", "hello store").unwrap();
        created.session.save(dir.to_str().unwrap()).unwrap();
        drop(created);

        let loaded = Session::load(dir.to_str().unwrap(), "pw", &secret).unwrap();
        assert_eq!(loaded.view().identity, identity);
        let history = loaded.history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].kind, "post");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reinstall_keeps_the_group_book() {
        use crate::talk::TalkNet;

        let secret = "44".repeat(32);
        let dir = std::env::temp_dir().join(format!("reedhold-circles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut alice = Session::create("pw", &secret).unwrap().session;
        let talk = TalkNet::open(1, &"00".repeat(32), &[alice.peer_hex()], None, Some(1)).unwrap();
        let group = talk.create_circle(&mut alice, "room").unwrap();
        alice.save(dir.to_str().unwrap()).unwrap();
        drop(alice);
        let loaded = Session::load(dir.to_str().unwrap(), "pw", &secret).unwrap();
        assert_eq!(
            loaded
                .circle(reedhold_core::ConversationId::from_hex(&group.id).unwrap())
                .unwrap()
                .name,
            "room"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
