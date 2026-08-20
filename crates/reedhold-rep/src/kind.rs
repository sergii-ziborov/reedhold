//! Like vs dislike vs endorse. Endorse spends more budget.

use crate::milli::Milli;

/// Reputation-bearing reaction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReactionKind {
    /// Low-risk interest.
    Like,
    /// Low-risk rejection.
    Dislike,
    /// Stakes curation reputation.
    Endorse,
    /// Says this content calls for harm.
    ///
    /// Deliberately not a wordlist. A list is evaded by spelling, punishes
    /// people quoting the thing they oppose, breaks across languages, and
    /// hands whoever writes it a lever over speech — which is the one power
    /// this protocol refuses to have. A report says nothing about words; it
    /// stakes the reporter's own standing on a judgement, and only counts once
    /// independent, unrelated accounts have made the same one.
    Report,
}

impl ReactionKind {
    /// Host-API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Dislike => "dislike",
            Self::Endorse => "endorse",
            Self::Report => "report",
        }
    }

    /// Parse a host-API name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "like" => Some(Self::Like),
            "dislike" => Some(Self::Dislike),
            "endorse" => Some(Self::Endorse),
            "report" => Some(Self::Report),
            _ => None,
        }
    }

    /// Influence units spent in the current epoch.
    #[must_use]
    pub const fn cost(self) -> u32 {
        match self {
            Self::Like | Self::Dislike => 1,
            Self::Endorse => 10,
            // A report is expensive to make, so nobody can spray them, and a
            // brigade exhausts its epoch budget long before it lands a blow.
            Self::Report => 25,
        }
    }

    /// Base weight before multipliers. Endorse is heavier.
    #[must_use]
    pub const fn base(self) -> Milli {
        match self {
            Self::Like | Self::Dislike => 1000,
            Self::Endorse => 4000,
            // Heavier than an endorsement, because agreeing that something
            // calls for harm is a stronger claim than liking it. The weight is
            // still multiplied by independence, so a cluster saying it a
            // thousand times says it once.
            Self::Report => 6000,
        }
    }

    /// Sign on content reputation. Dislike subtracts.
    #[must_use]
    pub const fn sign(self) -> i32 {
        match self {
            Self::Dislike | Self::Report => -1,
            Self::Like | Self::Endorse => 1,
        }
    }

    /// Whether making this claim puts the reporter's own standing at stake.
    ///
    /// A report that never attracts independent corroboration reads as
    /// brigading and costs the account that made it. That is what stops the
    /// tool from becoming a weapon against people you merely disagree with.
    #[must_use]
    pub const fn stakes_the_speaker(self) -> bool {
        matches!(self, Self::Endorse | Self::Report)
    }
}

#[cfg(test)]
mod tests {
    use super::ReactionKind;

    #[test]
    fn endorse_costs_more_than_a_like() {
        assert!(ReactionKind::Endorse.cost() > ReactionKind::Like.cost());
        assert!(ReactionKind::Endorse.base() > ReactionKind::Like.base());
    }

    #[test]
    fn a_report_is_the_heaviest_claim_and_the_dearest_to_make() {
        let report = ReactionKind::Report;
        assert!(report.base() > ReactionKind::Endorse.base());
        assert!(report.cost() > ReactionKind::Endorse.cost());
        assert_eq!(report.sign(), -1);
        assert!(report.stakes_the_speaker());
        assert!(!ReactionKind::Dislike.stakes_the_speaker());
        assert_eq!(ReactionKind::from_name("report"), Some(report));
    }
}
