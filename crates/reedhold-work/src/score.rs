//! Accumulated contribution. Not a token balance.

use crate::kind::WorkKind;
use crate::math::isqrt;

/// Per-node contribution history.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Score {
    /// Durable storage units.
    pub storage: u32,
    /// Reliable storage units.
    pub reliability: u32,
    /// Relay units.
    pub relay: u32,
    /// Repair units.
    pub repair: u32,
    /// Uptime units.
    pub uptime: u32,
    /// Distinct epochs seen.
    pub longevity: u32,
    /// Content units. Soft-capped.
    pub content: u32,
    /// Curation units. Soft-capped.
    pub curation: u32,
}

impl Score {
    /// Add `units` to the matching dimension.
    pub fn add(&mut self, kind: WorkKind, units: u32, reliable: bool) {
        let awarded = if reliable { units } else { units / 2 };
        match kind {
            WorkKind::Storage => {
                self.storage = self.storage.saturating_add(units);
                self.reliability = self.reliability.saturating_add(awarded);
            }
            WorkKind::Relay => self.relay = self.relay.saturating_add(awarded),
            WorkKind::Repair => self.repair = self.repair.saturating_add(awarded),
            WorkKind::Uptime => self.uptime = self.uptime.saturating_add(awarded),
            WorkKind::Content => self.content = self.content.saturating_add(awarded),
            WorkKind::Curation => self.curation = self.curation.saturating_add(awarded),
        }
    }

    /// Network weight. Social dimensions cannot outrun actual work.
    #[must_use]
    pub fn weight(self) -> u32 {
        let work = isqrt(self.storage)
            .saturating_add(isqrt(self.reliability))
            .saturating_add(isqrt(self.relay))
            .saturating_add(isqrt(self.repair).saturating_mul(2))
            .saturating_add(isqrt(self.uptime))
            .saturating_add(isqrt(self.longevity));
        let social = isqrt(self.content)
            .min(20)
            .saturating_add(isqrt(self.curation).min(20));
        work.saturating_add(social)
    }
}

#[cfg(test)]
mod tests {
    use super::Score;
    use crate::kind::WorkKind;

    #[test]
    fn social_cannot_outrun_repair() {
        let mut star = Score::default();
        star.add(WorkKind::Content, 100_000, true);
        let mut worker = Score::default();
        worker.add(WorkKind::Repair, 10_000, true);
        assert!(worker.weight() > star.weight());
    }
}
