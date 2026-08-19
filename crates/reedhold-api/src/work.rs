//! Host API for proof of contribution. Credits move; history does not.

use reedhold_core::{Digest32, Result};
use reedhold_work::{Book, WorkKind};
use serde::Serialize;

/// Contribution snapshot plus credit balance.
#[derive(Clone, Debug, Serialize)]
pub struct WorkView {
    /// Node hex.
    pub node: String,
    /// Network weight. Social dimensions are capped.
    pub weight: u32,
    /// Transferable sandbox credits.
    pub credits: u64,
    /// Repair units in the history.
    pub repair: u32,
    /// Whether `social` would make this node consensus-eligible.
    pub eligible: bool,
}

/// In-process contribution book.
pub struct WorkSession {
    book: Book,
}

impl WorkSession {
    /// Empty book.
    #[must_use]
    pub fn open() -> Self {
        Self { book: Book::new() }
    }

    /// Record work and mint credits (epoch-capped).
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Work`] on an unknown kind or zero units.
    pub fn record(
        &mut self,
        node_hex: &str,
        kind: &str,
        units: u32,
        epoch: u64,
        reliable: bool,
    ) -> Result<u32> {
        let kind =
            WorkKind::from_name(kind).ok_or(reedhold_core::Error::Work("unknown work kind"))?;
        self.book
            .record(Digest32::from_hex(node_hex)?, kind, units, epoch, reliable)
    }

    /// Score + credits. `social` is host-supplied reputation strength.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the hex id is invalid.
    pub fn view(&self, node_hex: &str, social: u32) -> Result<WorkView> {
        let node = Digest32::from_hex(node_hex)?;
        let score = self.book.score(node);
        Ok(WorkView {
            node: node.to_hex(),
            weight: score.weight(),
            credits: self.book.credits(node),
            repair: score.repair,
            eligible: self.book.eligible(node, social),
        })
    }

    /// Move credits. Contribution history stays with the sender.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Work`] when the balance is too small.
    pub fn transfer(&mut self, from_hex: &str, to_hex: &str, amount: u64) -> Result<()> {
        self.book.transfer(
            Digest32::from_hex(from_hex)?,
            Digest32::from_hex(to_hex)?,
            amount,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::WorkSession;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn popularity_is_not_consensus_and_credits_move() {
        let mut work = WorkSession::open();
        assert!(!work.view(&hex(9), 10_000).unwrap().eligible);
        work.record(&hex(1), "repair", 4000, 1, true).unwrap();
        assert!(work.view(&hex(1), 200).unwrap().eligible);
        let before = work.view(&hex(1), 0).unwrap().repair;
        work.transfer(&hex(1), &hex(2), 5).unwrap();
        assert_eq!(work.view(&hex(2), 0).unwrap().credits, 5);
        assert_eq!(work.view(&hex(1), 0).unwrap().repair, before);
        assert_eq!(work.view(&hex(2), 0).unwrap().repair, 0);
    }
}
