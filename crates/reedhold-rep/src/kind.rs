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
}

impl ReactionKind {
    /// Host-API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Dislike => "dislike",
            Self::Endorse => "endorse",
        }
    }

    /// Parse a host-API name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "like" => Some(Self::Like),
            "dislike" => Some(Self::Dislike),
            "endorse" => Some(Self::Endorse),
            _ => None,
        }
    }

    /// Influence units spent in the current epoch.
    #[must_use]
    pub const fn cost(self) -> u32 {
        match self {
            Self::Like | Self::Dislike => 1,
            Self::Endorse => 10,
        }
    }

    /// Base weight before multipliers. Endorse is heavier.
    #[must_use]
    pub const fn base(self) -> Milli {
        match self {
            Self::Like | Self::Dislike => 1000,
            Self::Endorse => 4000,
        }
    }

    /// Sign on content reputation. Dislike subtracts.
    #[must_use]
    pub const fn sign(self) -> i32 {
        match self {
            Self::Dislike => -1,
            Self::Like | Self::Endorse => 1,
        }
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
}
