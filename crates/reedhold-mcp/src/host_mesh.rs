//! Mesh fabric methods on the MCP host.

use crate::host::Host;
use reedhold_api::{MeshSession, RouteView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn mesh_open(
        &mut self,
        epoch: u64,
        prior: &str,
        candidates: &[String],
        company: Option<&str>,
    ) -> Result<()> {
        self.mesh = Some(MeshSession::open(epoch, prior, candidates, company, None)?);
        Ok(())
    }

    pub(crate) fn mesh_online(&mut self, peer: &str) -> Result<()> {
        self.mesh_mut()?.online(peer)
    }

    pub(crate) fn mesh_offline(&mut self, peer: &str) -> Result<()> {
        self.mesh_mut()?.offline(peer)
    }

    pub(crate) fn mesh_block(&mut self, peer: &str) -> Result<()> {
        self.mesh_mut()?.block(peer)
    }

    pub(crate) fn mesh_send(&mut self, from: &str, to: &str, payload: &str) -> Result<RouteView> {
        self.mesh_mut()?.send(from, to, payload)
    }

    pub(crate) fn mesh_drain(&mut self, peer: &str) -> Result<Vec<String>> {
        self.mesh_mut()?.drain(peer)
    }

    fn mesh_mut(&mut self) -> Result<&mut MeshSession> {
        self.mesh
            .as_mut()
            .ok_or(Error::Mesh("mesh fabric is not open"))
    }
}
