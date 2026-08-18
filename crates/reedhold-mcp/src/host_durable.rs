//! Durable grid methods on the MCP host.

use crate::host::Host;
use reedhold_api::{DurableSession, ObjectView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn durable_open(&mut self, holders: &[String], company: Option<&str>) -> Result<()> {
        self.durable = Some(DurableSession::open(holders, company)?);
        Ok(())
    }

    pub(crate) fn durable_put(&mut self, payload: &str, tier: &str) -> Result<ObjectView> {
        self.durable_mut()?.put(payload, tier)
    }

    pub(crate) fn durable_get(&self, id: &str) -> Result<String> {
        self.durable()?.get(id)
    }

    pub(crate) fn durable_kill(&mut self, holder: &str) -> Result<()> {
        self.durable_mut()?.kill(holder)
    }

    pub(crate) fn durable_repair(&mut self, id: &str) -> Result<ObjectView> {
        self.durable_mut()?.repair(id)
    }

    fn durable(&self) -> Result<&DurableSession> {
        self.durable
            .as_ref()
            .ok_or(Error::Storage("durable grid is not open"))
    }

    fn durable_mut(&mut self) -> Result<&mut DurableSession> {
        self.durable
            .as_mut()
            .ok_or(Error::Storage("durable grid is not open"))
    }
}
