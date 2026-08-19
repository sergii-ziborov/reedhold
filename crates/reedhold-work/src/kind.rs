//! Kinds of network work. Repair is worth more than idle content.

/// One contribution dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorkKind {
    /// Contracted durable bytes.
    Storage,
    /// Store-and-forward / bandwidth.
    Relay,
    /// Shard reconstruction.
    Repair,
    /// Availability.
    Uptime,
    /// Published content. Soft-capped in the weight.
    Content,
    /// Curation. Soft-capped in the weight.
    Curation,
}

impl WorkKind {
    /// Host-API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Storage => "storage",
            Self::Relay => "relay",
            Self::Repair => "repair",
            Self::Uptime => "uptime",
            Self::Content => "content",
            Self::Curation => "curation",
        }
    }

    /// Parse a host-API name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "storage" => Some(Self::Storage),
            "relay" => Some(Self::Relay),
            "repair" => Some(Self::Repair),
            "uptime" => Some(Self::Uptime),
            "content" => Some(Self::Content),
            "curation" => Some(Self::Curation),
            _ => None,
        }
    }

    /// Credits per work unit. Repair is the most expensive to fake cheaply.
    #[must_use]
    pub const fn rate(self) -> u32 {
        match self {
            Self::Repair => 4,
            Self::Relay => 2,
            Self::Storage | Self::Uptime | Self::Content | Self::Curation => 1,
        }
    }

    /// Whether this dimension is social (must not dominate consensus).
    #[must_use]
    pub const fn is_social(self) -> bool {
        matches!(self, Self::Content | Self::Curation)
    }
}

#[cfg(test)]
mod tests {
    use super::WorkKind;

    #[test]
    fn repair_pays_more_than_content() {
        assert!(WorkKind::Repair.rate() > WorkKind::Content.rate());
        assert!(WorkKind::Content.is_social());
        assert!(!WorkKind::Repair.is_social());
    }
}
