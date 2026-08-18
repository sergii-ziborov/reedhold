//! Durability classes. Not every byte is equally permanent.

/// Protocol durability class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DurabilityTier {
    /// Identity, recovery, device revocation, group heads.
    CriticalIdentity,
    /// Guaranteed personal text and social-graph essentials.
    PersonalHistory,
    /// Public posts. Preservation is adaptive.
    PublicSocial,
    /// Large media. Must fund its own storage.
    LargeMedia,
}

impl DurabilityTier {
    /// Stable wire tag.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::CriticalIdentity => 0,
            Self::PersonalHistory => 1,
            Self::PublicSocial => 2,
            Self::LargeMedia => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DurabilityTier;

    #[test]
    fn identity_outranks_media() {
        let identity = DurabilityTier::CriticalIdentity;
        let media = DurabilityTier::LargeMedia;
        assert!(identity < media);
        assert!(identity.as_u8() < media.as_u8());
    }
}
