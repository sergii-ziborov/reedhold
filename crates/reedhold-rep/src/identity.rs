//! Orthogonal identity dimensions. UI may fold them into one Strength.

/// Non-transferable identity reputation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentityRep {
    /// Meaningful history, not idle age.
    pub continuity: u32,
    /// Social trust.
    pub social: u32,
    /// Content quality.
    pub content: u32,
    /// Curation quality.
    pub curation: u32,
    /// Network contribution.
    pub contribution: u32,
    /// Moderation quality.
    pub moderation: u32,
}

impl IdentityRep {
    /// Aggregate strength 0..=10000. UI number, not a token balance.
    #[must_use]
    pub fn strength(self) -> u32 {
        let sum = u64::from(self.continuity)
            + u64::from(self.social)
            + u64::from(self.content)
            + u64::from(self.curation)
            + u64::from(self.contribution)
            + u64::from(self.moderation);
        u32::try_from(sum / 6).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::IdentityRep;

    #[test]
    fn idle_age_is_not_trust() {
        let idle = IdentityRep {
            continuity: 8000,
            ..IdentityRep::default()
        };
        assert!(idle.strength() < 2000);
    }
}
