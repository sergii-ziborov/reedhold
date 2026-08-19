//! In-process session used by every host.

use crate::view::{AccountView, EventView, ManifestView};
use reedhold_core::{ConversationId, Error, IdentityId, NetworkId, Result, decode_hex, decode32};
use reedhold_event::{EventKind, SignedEvent, open_message, seal_message};
use reedhold_protocol::{Account, Circle, create_account, restore_account};
use reedhold_recovery::KdfParams;
use reedhold_store::StoredEvent;
use std::collections::BTreeMap;

/// Freshly created session plus the first manifest.
pub struct Created {
    /// Live session.
    pub session: Session,
    /// First recovery manifest.
    pub manifest: ManifestView,
}

/// Unlocked account. Hosts keep this in memory and persist via `LocalStore`.
pub struct Session {
    pub(crate) account: Account,
    pub(crate) log: Vec<StoredEvent>,
    pub(crate) circles: BTreeMap<ConversationId, Circle>,
    pubs: BTreeMap<IdentityId, [u8; 32]>,
}

impl Session {
    /// Create a new identity. `device_secret_hex` must be 32 bytes hex.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when entropy, KDF, or hex decoding fails.
    pub fn create(password: &str, device_secret_hex: &str) -> Result<Created> {
        let device = decode32(device_secret_hex)?;
        let created = create_account(
            NetworkId::DEV,
            password.as_bytes(),
            &device,
            KdfParams::TEST,
        )?;
        let manifest = manifest_view(created.account.manifest())?;
        Ok(Created {
            session: Self::from_parts(created.account, Vec::new()),
            manifest,
        })
    }

    /// Restore from a stored manifest hex.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when the password or bytes are wrong.
    pub fn restore(manifest_hex: &str, password: &str, device_secret_hex: &str) -> Result<Self> {
        let device = decode32(device_secret_hex)?;
        let bytes = decode_hex(manifest_hex)?;
        let manifest = reedhold_recovery::RecoveryManifest::decode(&bytes, NetworkId::DEV)?;
        Ok(Self::from_parts(
            restore_account(&manifest, password.as_bytes(), &device)?,
            Vec::new(),
        ))
    }

    /// Public snapshot.
    #[must_use]
    pub fn view(&self) -> AccountView {
        AccountView::from_account(&self.account)
    }

    /// Latest manifest hex.
    ///
    /// # Errors
    ///
    /// Returns a codec error when the manifest cannot be encoded.
    pub fn manifest(&self) -> Result<ManifestView> {
        manifest_view(self.account.manifest())
    }

    /// Sign a UTF-8 payload.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when `kind` is unknown.
    pub fn emit(&mut self, kind: &str, payload: &str) -> Result<EventView> {
        let kind = EventKind::from_name(kind).ok_or(Error::Event("unknown event kind"))?;
        let body = payload.as_bytes();
        let event = self.account.emit(kind, body)?;
        self.push_log(&event, body)?;
        EventView::from_event(&event, body)
    }

    /// Verify a hex-encoded signed event against this session's device key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the signature is invalid.
    pub fn verify(&self, event_hex: &str) -> Result<EventView> {
        let bytes = decode_hex(event_hex)?;
        let event =
            SignedEvent::decode_verify(&bytes, NetworkId::DEV, &self.account.device_public())?;
        EventView::from_event(&event, &[])
    }

    /// Change the vault password. Identity is unchanged.
    ///
    /// # Errors
    ///
    /// Returns a recovery error when sealing fails.
    pub fn change_password(&mut self, password: &str) -> Result<ManifestView> {
        let manifest = self
            .account
            .change_password(password.as_bytes(), KdfParams::TEST)?;
        manifest_view(&manifest)
    }

    /// Seal plaintext and emit it as a direct message.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when the conversation key is wrong length.
    pub fn emit_sealed(
        &mut self,
        conversation_key_hex: &str,
        plaintext: &str,
    ) -> Result<EventView> {
        let key = decode32(conversation_key_hex)?;
        let envelope = seal_message(&key, plaintext.as_bytes())?;
        let body = envelope.encode()?;
        let event = self.account.emit(EventKind::DirectMessage, &body)?;
        self.push_log(&event, &body)?;
        EventView::from_event(&event, &body)
    }

    pub(crate) fn from_parts(account: Account, log: Vec<StoredEvent>) -> Self {
        Self {
            account,
            log,
            circles: BTreeMap::new(),
            pubs: BTreeMap::new(),
        }
    }

    /// Identity digest hex. The in-process talk net uses this as the peer id.
    #[must_use]
    pub fn peer_hex(&self) -> String {
        self.account.identity().to_hex()
    }

    pub(crate) fn remember_circle(&mut self, circle: Circle) {
        self.circles.insert(circle.id, circle);
    }

    pub(crate) fn circle(&self, id: ConversationId) -> Result<&Circle> {
        self.circles.get(&id).ok_or(Error::Event("unknown group"))
    }

    pub(crate) fn circle_mut(&mut self, id: ConversationId) -> Result<&mut Circle> {
        self.circles
            .get_mut(&id)
            .ok_or(Error::Event("unknown group"))
    }

    pub(crate) fn forget_circle(&mut self, id: ConversationId) {
        self.circles.remove(&id);
    }

    pub(crate) fn remember_pub(&mut self, id: IdentityId, public: [u8; 32]) {
        self.pubs.insert(id, public);
    }

    pub(crate) fn lookup_pub(&self, id: IdentityId) -> Result<[u8; 32]> {
        self.pubs
            .get(&id)
            .copied()
            .ok_or(Error::Event("unknown member messaging key"))
    }

    pub(crate) fn push_log(&mut self, event: &SignedEvent, body: &[u8]) -> Result<()> {
        self.log.push(StoredEvent {
            encoded: event.encode()?,
            body: body.to_vec(),
        });
        Ok(())
    }

    /// Open a sealed envelope hex with a conversation key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the key or envelope is wrong.
    pub fn open_sealed(conversation_key_hex: &str, envelope_hex: &str) -> Result<String> {
        let key = decode32(conversation_key_hex)?;
        let envelope = reedhold_event::MessageEnvelope::decode(&decode_hex(envelope_hex)?)?;
        let plain = open_message(&key, &envelope)?;
        String::from_utf8(plain).map_err(|_| Error::Event("sealed payload is not utf-8"))
    }
}

fn manifest_view(manifest: &reedhold_recovery::RecoveryManifest) -> Result<ManifestView> {
    Ok(ManifestView {
        identity: manifest.identity.to_uri(),
        epoch: manifest.epoch,
        manifest_hex: reedhold_core::encode_hex(&manifest.encode()?),
    })
}

#[cfg(test)]
mod tests {
    use super::Session;

    #[test]
    fn host_round_trip() {
        let secret = "11".repeat(32);
        let mut created = Session::create("pw", &secret).unwrap();
        let identity = created.session.view().identity;
        let event = created.session.emit("post", "hello").unwrap();
        let restored = Session::restore(&created.manifest.manifest_hex, "pw", &secret).unwrap();
        assert_eq!(restored.view().identity, identity);
        restored.verify(&event.event_hex).unwrap();
    }
}
