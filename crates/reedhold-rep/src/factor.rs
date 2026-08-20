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

/// Objections past this point stop adding up.
const OBJECTION_KNEE: u32 = 8_000;

/// Aggregate negative weight, saturated. Positive weight is never saturated.
///
/// Independence stops a cluster, but it cannot stop a real majority: a hundred
/// thousand unrelated people piling on a minority are independent by every
/// graph measure there is. Any rule that lets volume decide a verdict is
/// therefore a weapon pointed at exactly the communities that are outnumbered
/// by definition — which is most of the ones that get piled on.
///
/// So the curve goes concave past the knee. The tenth independent objection
/// counts; the ten-thousandth barely moves anything. A crowd keeps its voice
/// and loses its ability to convert headcount into a judgement, while support
/// for the person being piled on stays linear and can still answer it.
#[must_use]
pub fn saturated_objection(raw: u32) -> u32 {
    if raw <= OBJECTION_KNEE {
        return raw;
    }
    let excess = raw - OBJECTION_KNEE;
    OBJECTION_KNEE.saturating_add(isqrt(excess.saturating_mul(4)))
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
    fn a_crowd_cannot_convert_headcount_into_a_verdict() {
        use super::saturated_objection;
        let modest = saturated_objection(4_000);
        let large = saturated_objection(40_000);
        let enormous = saturated_objection(4_000_000);

        assert_eq!(modest, 4_000, "below the knee nothing is dampened");
        // Ten times the pile-on is nowhere near ten times the effect, and a
        // thousand times is barely more than ten times.
        assert!(large < modest * 3);
        assert!(enormous < large * 2);
        assert!(
            enormous < 20_000,
            "four million objections stayed under twenty thousand: {enormous}"
        );
    }

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
