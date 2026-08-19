//! One in-memory session for the agent process.

use reedhold_api::{
    AccountView, ChainSession, DurableSession, EventView, ManifestView, MarketSession, MeshSession,
    RepSession, Session, TalkNet,
};
use reedhold_core::{Error, Result};

/// Mutable host state shared by every MCP tool.
#[derive(Default)]
pub struct Host {
    pub(crate) session: Option<Session>,
    pub(crate) mesh: Option<MeshSession>,
    pub(crate) durable: Option<DurableSession>,
    pub(crate) talk: Option<TalkNet>,
    pub(crate) chain: Option<ChainSession>,
    pub(crate) rep: Option<RepSession>,
    pub(crate) ads: Option<MarketSession>,
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

    pub(crate) fn save(&self, dir: &str) -> Result<()> {
        self.session()?.save(dir)
    }

    pub(crate) fn load(
        &mut self,
        dir: &str,
        password: &str,
        device_secret: &str,
    ) -> Result<AccountView> {
        let session = Session::load(dir, password, device_secret)?;
        let view = session.view();
        self.session = Some(session);
        Ok(view)
    }

    pub(crate) fn split_recovery(
        &self,
        threshold: u8,
        total: u8,
    ) -> Result<Vec<reedhold_api::ShareView>> {
        self.session()?.split_recovery(threshold, total)
    }

    pub(crate) fn combine_recovery(
        &mut self,
        shares: &[reedhold_api::ShareView],
        threshold: u8,
        password: &str,
        device_secret: &str,
    ) -> Result<(AccountView, ManifestView)> {
        let (session, manifest) =
            reedhold_api::session_from_shares(shares, threshold, password, device_secret)?;
        let view = session.view();
        self.session = Some(session);
        Ok((view, manifest))
    }

    pub(crate) fn session(&self) -> Result<&Session> {
        self.session
            .as_ref()
            .ok_or(Error::Identity("no unlocked session"))
    }

    pub(crate) fn session_mut(&mut self) -> Result<&mut Session> {
        self.session
            .as_mut()
            .ok_or(Error::Identity("no unlocked session"))
    }
}
