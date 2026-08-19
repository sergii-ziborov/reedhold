//! Contribution book: scores stay, credits move.

use crate::kind::WorkKind;
use crate::score::Score;
use reedhold_core::{Digest32, Error, Result};
use std::collections::{BTreeMap, BTreeSet};

const EPOCH_CREDIT_CAP: u32 = 10_000;
const ELIGIBLE_MIN: u32 = 20;

/// In-process proof-of-contribution ledger.
#[derive(Clone, Debug, Default)]
pub struct Book {
    scores: BTreeMap<Digest32, Score>,
    credits: BTreeMap<Digest32, u64>,
    paid: BTreeMap<(Digest32, u64), u32>,
    epochs: BTreeMap<Digest32, BTreeSet<u64>>,
}

impl Book {
    /// Empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record work. Mints credits up to the epoch cap. Does not move history.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Work`] when `units` is zero.
    pub fn record(
        &mut self,
        node: Digest32,
        kind: WorkKind,
        units: u32,
        epoch: u64,
        reliable: bool,
    ) -> Result<u32> {
        if units == 0 {
            return Err(Error::Work("work units must be positive"));
        }
        let score = self.scores.entry(node).or_default();
        score.add(kind, units, reliable);
        if self.epochs.entry(node).or_default().insert(epoch) {
            let seen = u32::try_from(self.epochs[&node].len()).unwrap_or(u32::MAX);
            if let Some(score) = self.scores.get_mut(&node) {
                score.longevity = seen;
            }
        }
        let rate = kind.rate();
        let milli = if reliable { 1000 } else { 500 };
        let raw = units.saturating_mul(rate).saturating_mul(milli) / 1000;
        let already = self.paid.get(&(node, epoch)).copied().unwrap_or(0);
        let room = EPOCH_CREDIT_CAP.saturating_sub(already);
        let minted = raw.min(room);
        *self.paid.entry((node, epoch)).or_insert(0) = already.saturating_add(minted);
        let wallet = self.credits.entry(node).or_insert(0);
        *wallet = wallet.saturating_add(u64::from(minted));
        Ok(minted)
    }

    /// Contribution snapshot.
    #[must_use]
    pub fn score(&self, node: Digest32) -> Score {
        self.scores.get(&node).copied().unwrap_or_default()
    }

    /// Transferable credit balance.
    #[must_use]
    pub fn credits(&self, node: Digest32) -> u64 {
        self.credits.get(&node).copied().unwrap_or(0)
    }

    /// Move credits. History stays with `from`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Work`] when the balance is too small.
    pub fn transfer(&mut self, from: Digest32, to: Digest32, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(Error::Work("transfer amount must be positive"));
        }
        let bal = self.credits(from);
        if bal < amount {
            return Err(Error::Work("insufficient credits"));
        }
        *self.credits.entry(from).or_insert(0) = bal - amount;
        let dest = self.credits.entry(to).or_insert(0);
        *dest = dest.saturating_add(amount);
        Ok(())
    }

    /// Consensus eligibility. `social` is host-supplied reputation strength.
    /// Popularity alone is never enough, and credits buy nothing here.
    #[must_use]
    pub fn eligible(&self, node: Digest32, social: u32) -> bool {
        let work = self.score(node).consensus_weight();
        if work < u64::from(ELIGIBLE_MIN) {
            return false;
        }
        work >= u64::from(crate::math::isqrt(social))
    }
}

#[cfg(test)]
mod tests {
    use super::Book;
    use crate::kind::WorkKind;
    use reedhold_core::Digest32;

    fn node(byte: u8) -> Digest32 {
        Digest32::from_bytes([byte; 32])
    }

    #[test]
    fn credits_move_and_history_does_not() {
        let mut book = Book::new();
        book.record(node(1), WorkKind::Repair, 100, 1, true)
            .unwrap();
        let score = book.score(node(1)).repair;
        book.transfer(node(1), node(2), 10).unwrap();
        assert_eq!(book.credits(node(2)), 10);
        assert_eq!(book.score(node(1)).repair, score);
        assert_eq!(book.score(node(2)).repair, 0);
        assert!(book.transfer(node(2), node(1), 9999).is_err());
    }

    #[test]
    fn celebrity_without_work_is_not_eligible() {
        let mut book = Book::new();
        assert!(!book.eligible(node(9), 10_000));
        book.record(node(3), WorkKind::Repair, 5_000, 1, true)
            .unwrap();
        assert!(book.eligible(node(3), 100));
        book.record(node(4), WorkKind::Storage, 80, 1, false)
            .unwrap();
        let reliable = {
            let mut other = Book::new();
            other
                .record(node(4), WorkKind::Storage, 80, 1, true)
                .unwrap();
            other.credits(node(4))
        };
        assert!(book.credits(node(4)) < reliable);
    }
}
