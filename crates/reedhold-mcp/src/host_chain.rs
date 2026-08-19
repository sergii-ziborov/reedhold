//! Compact-chain methods on the MCP host.

use crate::host::Host;
use reedhold_api::{ChainSession, HeaderView, ProofView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn chain_open(&mut self) -> Result<()> {
        self.chain = Some(ChainSession::open()?);
        Ok(())
    }

    pub(crate) fn chain_commit(
        &mut self,
        epoch: u64,
        identity: &str,
        groups: &str,
        storage: &str,
    ) -> Result<HeaderView> {
        self.chain_mut()?.commit(epoch, identity, groups, storage)
    }

    pub(crate) fn chain_head(&self) -> Result<HeaderView> {
        Ok(self.chain()?.head())
    }

    pub(crate) fn chain_headers(&self) -> Result<Vec<HeaderView>> {
        Ok(self.chain()?.headers())
    }

    pub(crate) fn chain_prove(&self, leaves: &[String], index: u32) -> Result<ProofView> {
        self.chain()?.prove(leaves, index)
    }

    pub(crate) fn chain_verify(
        &self,
        leaf: &str,
        root: &str,
        index: u32,
        siblings: &[String],
    ) -> Result<bool> {
        self.chain()?.verify(leaf, root, index, siblings)
    }

    fn chain(&self) -> Result<&ChainSession> {
        self.chain.as_ref().ok_or(Error::Chain("chain is not open"))
    }

    fn chain_mut(&mut self) -> Result<&mut ChainSession> {
        self.chain.as_mut().ok_or(Error::Chain("chain is not open"))
    }
}
