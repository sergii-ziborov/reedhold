//! Directory save/load. The host picks the path (app sandbox, not our home).

use crate::session::Session;
use crate::view::EventView;
use reedhold_core::{NetworkId, Result};
use reedhold_event::SignedEvent;
use reedhold_protocol::restore_account;
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
        store.save_events(&self.log)
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
        Ok(Self::from_parts(account, raw))
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
}
