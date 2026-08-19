//! In-process header log. Still not a message store.

use crate::header::Header;
use crate::roots::EpochRoots;
use reedhold_core::{NetworkId, Result};

/// Append-only compact chain.
#[derive(Clone, Debug)]
pub struct Ledger {
    headers: Vec<Header>,
}

impl Ledger {
    /// Genesis header for `network`.
    #[must_use]
    pub fn genesis(network: NetworkId) -> Self {
        Self {
            headers: vec![Header::genesis(network)],
        }
    }

    /// Latest header.
    #[must_use]
    pub fn head(&self) -> Header {
        self.headers
            .last()
            .copied()
            .unwrap_or_else(|| Header::genesis(NetworkId::DEV))
    }

    /// Commit subtree roots. Callers pass 32-byte roots, never message bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Chain`] if the successor cannot be linked.
    pub fn commit(&mut self, epoch: u64, roots: EpochRoots) -> Result<Header> {
        let header = self.head().successor(epoch, roots);
        self.headers.push(header);
        Ok(header)
    }

    /// Last `limit` headers, oldest first.
    #[must_use]
    pub fn window(&self, limit: usize) -> &[Header] {
        let start = self.headers.len().saturating_sub(limit);
        &self.headers[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::Ledger;
    use crate::roots::EpochRoots;
    use reedhold_core::{Digest32, NetworkId};

    #[test]
    fn commit_links_prev_and_never_takes_bytes() {
        let mut ledger = Ledger::genesis(NetworkId::DEV);
        let mut roots = EpochRoots::empty();
        roots.identity = Digest32::from_bytes([4; 32]);
        let header = ledger.commit(1, roots).unwrap();
        assert_eq!(header.height, 1);
        assert_eq!(
            header.prev,
            crate::header::Header::genesis(NetworkId::DEV).hash()
        );
        assert_eq!(header.roots.identity, roots.identity);
    }
}
