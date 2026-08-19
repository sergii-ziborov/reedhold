//! Saturating multipliers. Linear `weight = liker_rep` is forbidden.

use crate::identity::IdentityRep;
use crate::milli::{Milli, ONE, isqrt};

const REF_REP: u32 = 1000;
const FACTOR_CAP: Milli = 2000;
const AFTER_BUDGET: Milli = 200;

/// `sqrt(R)/sqrt(R_ref)` capped. New accounts are cheap.
#[must_use]
pub fn rep_factor(reputation: u32) -> Milli {
    let numer = isqrt(reputation.saturating_add(1));
    let denom = isqrt(REF_REP + 1).max(1);
    let scaled = u64::from(numer) * u64::from(ONE) / u64::from(denom);
    u32::try_from(scaled).unwrap_or(u32::MAX).min(FACTOR_CAP)
}

/// Same-cluster reactions share the pie. First is full; the ten-thousandth is tiny.
#[must_use]
pub fn independence(prior_same_cluster: u32) -> Milli {
    let denom = isqrt(prior_same_cluster.saturating_add(1)).max(1);
    ONE / denom
}

/// Topic skill from the content dimension when a topic is set. Else global.
#[must_use]
pub fn topic_factor(identity: &IdentityRep, topic_set: bool) -> Milli {
    if topic_set {
        500 + identity.content / 20
    } else {
        ONE
    }
}

/// Neutral curator starts at 1.0; high curation quality lifts slightly.
#[must_use]
pub fn curator_factor(identity: &IdentityRep) -> Milli {
    750 + identity.curation / 40
}

/// After the epoch cap, likes still land but barely move reputation.
#[must_use]
pub fn budget_factor(spent: u32, cap: u32) -> Milli {
    if cap == 0 || spent >= cap {
        return AFTER_BUDGET;
    }
    let used = u64::from(spent) * u64::from(ONE) / u64::from(cap);
    let decay = u32::try_from(used * 8 / 10).unwrap_or(u32::MAX);
    ONE.saturating_sub(decay).max(AFTER_BUDGET)
}

/// Weekly units: a floor plus sqrt of mature strength.
#[must_use]
pub fn epoch_budget(strength: u32) -> u32 {
    100 + isqrt(strength)
}

#[cfg(test)]
mod tests {
    use super::{budget_factor, independence, rep_factor};
    use crate::milli::ONE;

    #[test]
    fn cluster_and_budget_saturate() {
        assert!(rep_factor(0) < 100);
        assert!(rep_factor(1000) >= 900);
        assert!(rep_factor(100_000) <= 2000);
        assert_eq!(independence(0), ONE);
        assert!(independence(10_000) < 50);
        assert_eq!(budget_factor(0, 100), ONE);
        assert_eq!(budget_factor(100, 100), 200);
    }
}
