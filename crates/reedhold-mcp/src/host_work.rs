//! Proof-of-contribution methods on the MCP host.

use crate::host::Host;
use reedhold_api::{WorkSession, WorkView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn work_open(&mut self) {
        self.work = Some(WorkSession::open());
    }

    pub(crate) fn work_record(
        &mut self,
        node: &str,
        kind: &str,
        units: u32,
        epoch: u64,
        reliable: bool,
    ) -> Result<u32> {
        self.work_mut()?.record(node, kind, units, epoch, reliable)
    }

    pub(crate) fn work_view(&self, node: &str, social: u32) -> Result<WorkView> {
        self.work()?.view(node, social)
    }

    pub(crate) fn work_transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<()> {
        self.work_mut()?.transfer(from, to, amount)
    }

    fn work(&self) -> Result<&WorkSession> {
        self.work
            .as_ref()
            .ok_or(Error::Work("contribution book is not open"))
    }

    fn work_mut(&mut self) -> Result<&mut WorkSession> {
        self.work
            .as_mut()
            .ok_or(Error::Work("contribution book is not open"))
    }
}
