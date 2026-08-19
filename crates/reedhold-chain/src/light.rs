//! Bounded header window. A phone stores this, not the full chain.

use crate::header::Header;
use reedhold_core::{Error, Result};
use std::collections::VecDeque;

/// Recent headers a consumer node keeps.
pub const HEADER_WINDOW: usize = 64;

/// Light client: follow a compact header chain, refuse rollbacks.
#[derive(Clone, Debug)]
pub struct LightClient {
    headers: VecDeque<Header>,
    cap: usize,
}

impl LightClient {
    /// Empty window with the protocol cap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: VecDeque::new(),
            cap: HEADER_WINDOW,
        }
    }

    /// Latest followed header.
    #[must_use]
    pub fn head(&self) -> Option<Header> {
        self.headers.back().copied()
    }

    /// Headers currently retained, oldest first.
    #[must_use]
    pub fn window(&self) -> Vec<Header> {
        self.headers.iter().copied().collect()
    }

    /// Accept `header` if it extends the current head. Rejects forks at the same height.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] on rollback or a broken prev link.
    pub fn follow(&mut self, header: Header) -> Result<()> {
        match self.head() {
            None => {
                if header.height != 0 {
                    return Err(Error::Chain("light client must start at genesis"));
                }
            }
            Some(head) => {
                if header.height <= head.height {
                    return Err(Error::Chain("rollback"));
                }
                if header.height != head.height.saturating_add(1) || header.prev != head.hash() {
                    return Err(Error::Chain("broken header chain"));
                }
            }
        }
        self.headers.push_back(header);
        while self.headers.len() > self.cap {
            self.headers.pop_front();
        }
        Ok(())
    }
}

impl Default for LightClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{HEADER_WINDOW, LightClient};
    use crate::header::Header;
    use crate::roots::EpochRoots;
    use reedhold_core::NetworkId;

    #[test]
    fn window_is_bounded_and_rollback_is_rejected() {
        let mut light = LightClient::new();
        let mut header = Header::genesis(NetworkId::DEV);
        light.follow(header).unwrap();
        for epoch in 1..=HEADER_WINDOW + 8 {
            header = header.successor(u64::try_from(epoch).unwrap_or(0), EpochRoots::empty());
            light.follow(header).unwrap();
        }
        assert_eq!(light.window().len(), HEADER_WINDOW);
        assert!(light.follow(Header::genesis(NetworkId::DEV)).is_err());
        let fork = Header::genesis(NetworkId::DEV).successor(1, EpochRoots::empty());
        assert!(light.follow(fork).is_err());
    }
}
