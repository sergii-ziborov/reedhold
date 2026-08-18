//! One in-memory session for the agent process.

use reedhold_api::{AccountView, EventView, ManifestView, Session};
use reedhold_core::{Error, Result};

/// Mutable host state shared by every MCP tool.
#[derive(Default)]
pub struct Host {
    session: Option<Session>,
}

impl Host {
    pub(crate) fn create(
        &mut self,
        password: &str,
        device_secret: &str,
    ) -> Result<(AccountView, ManifestView)> {
        let created = Session::create(password, device_secret)?;
        let view = created.session.view();
        let manifest = created.manifest;
        self.session = Some(created.session);
        Ok((view, manifest))
    }

    pub(crate) fn restore(
        &mut self,
        manifest_hex: &str,
        password: &str,
        device_secret: &str,
    ) -> Result<AccountView> {
        let session = Session::restore(manifest_hex, password, device_secret)?;
        let view = session.view();
        self.session = Some(session);
        Ok(view)
    }

    pub(crate) fn view(&self) -> Result<AccountView> {
        Ok(self.session()?.view())
    }

    pub(crate) fn emit(&mut self, kind: &str, payload: &str) -> Result<EventView> {
        self.session_mut()?.emit(kind, payload)
    }

    pub(crate) fn verify(&self, event_hex: &str) -> Result<EventView> {
        self.session()?.verify(event_hex)
    }

    pub(crate) fn change_password(&mut self, password: &str) -> Result<ManifestView> {
        self.session_mut()?.change_password(password)
    }

    pub(crate) fn emit_sealed(
        &mut self,
        conversation_key: &str,
        plaintext: &str,
    ) -> Result<EventView> {
        self.session_mut()?.emit_sealed(conversation_key, plaintext)
    }

    fn session(&self) -> Result<&Session> {
        self.session
            .as_ref()
            .ok_or(Error::Identity("no unlocked session"))
    }

    fn session_mut(&mut self) -> Result<&mut Session> {
        self.session
            .as_mut()
            .ok_or(Error::Identity("no unlocked session"))
    }
}
