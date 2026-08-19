//! Reputation methods on the MCP host.

use crate::host::Host;
use reedhold_api::{ContentScoreView, IdentityScoreView, ReactionView, RepSession};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn rep_open(&mut self) {
        self.rep = Some(RepSession::open());
    }

    pub(crate) fn rep_seed(
        &mut self,
        identity: &str,
        continuity: u32,
        social: u32,
        content: u32,
        curation: u32,
    ) -> Result<IdentityScoreView> {
        self.rep_mut()?
            .seed(identity, continuity, social, content, curation)
    }

    pub(crate) fn rep_react(
        &mut self,
        author: &str,
        target: &str,
        kind: &str,
        cluster: &str,
        now: u64,
    ) -> Result<ReactionView> {
        self.rep_mut()?.react(author, target, kind, cluster, now)
    }

    pub(crate) fn rep_identity(&self, identity: &str, now: u64) -> Result<IdentityScoreView> {
        self.rep()?.identity(identity, now)
    }

    pub(crate) fn rep_content(&self, target: &str, now: u64) -> Result<ContentScoreView> {
        self.rep()?.content(target, now)
    }

    pub(crate) fn rep_transfer(from: &str, to: &str, amount: u32) -> Result<()> {
        RepSession::transfer(from, to, amount)
    }

    fn rep(&self) -> Result<&RepSession> {
        self.rep
            .as_ref()
            .ok_or(Error::Reputation("reputation book is not open"))
    }

    fn rep_mut(&mut self) -> Result<&mut RepSession> {
        self.rep
            .as_mut()
            .ok_or(Error::Reputation("reputation book is not open"))
    }
}
