//! Piecewise maturity. Instant reactions are almost weightless.

use crate::kind::ReactionKind;
use crate::milli::{Milli, ONE};

/// Seconds in one minute.
const MINUTE: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86_400;
const WEEK: u64 = 7 * DAY;
const MONTH: u64 = 30 * DAY;
const YEAR: u64 = 365 * DAY;

/// Spec UX table, milli. Endorse ages at half speed.
#[must_use]
pub fn maturity(age_secs: u64, kind: ReactionKind) -> Milli {
    let age = match kind {
        ReactionKind::Endorse => age_secs / 2,
        ReactionKind::Like | ReactionKind::Dislike => age_secs,
    };
    interpolate(age)
}

fn interpolate(age: u64) -> Milli {
    let points: [(u64, Milli); 7] = [
        (0, 0),
        (MINUTE, 30),
        (HOUR, 100),
        (DAY, 250),
        (WEEK, 550),
        (MONTH, 800),
        (YEAR, ONE),
    ];
    if age >= YEAR {
        return ONE;
    }
    for window in points.windows(2) {
        let (left_t, left_v) = window[0];
        let (right_t, right_v) = window[1];
        if age <= right_t {
            let span = right_t.saturating_sub(left_t).max(1);
            let delta = age.saturating_sub(left_t);
            let rise = u64::from(right_v.saturating_sub(left_v));
            let add = rise.saturating_mul(delta) / span;
            return left_v.saturating_add(u32::try_from(add).unwrap_or(u32::MAX));
        }
    }
    ONE
}

#[cfg(test)]
mod tests {
    use super::maturity;
    use crate::kind::ReactionKind;
    use crate::milli::ONE;

    #[test]
    fn instant_is_cheap_and_a_week_settles() {
        let now = maturity(0, ReactionKind::Like);
        let week = maturity(7 * 86_400, ReactionKind::Like);
        assert!(now < 50);
        assert!(week > 500);
        assert_eq!(maturity(365 * 86_400, ReactionKind::Like), ONE);
        assert!(maturity(7 * 86_400, ReactionKind::Endorse) < week);
    }
}
