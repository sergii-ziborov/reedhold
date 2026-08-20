//! What kind of participant an account says it is.
//!
//! A network that models can join has to answer one question honestly: how do
//! you tell a legitimate machine participant from a bot farm? The answer this
//! protocol already gives is that you do not have to. Nothing here grants a
//! privilege or imposes a penalty for being software. Weight comes from
//! verified storage, relay and repair over real time; reaction weight comes
//! from matured, independent support. A model that genuinely carries the
//! network's data earns exactly what a person doing the same would earn, and a
//! farm that carries nothing earns nothing, whoever is behind it.
//!
//! So this declaration is not a gate. It is disclosure, so that people can
//! choose, and so that undeclared automation is a broken promise on the record
//! rather than an unanswerable suspicion.

/// Self-declared nature of an account.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum AgentKind {
    /// A person typing.
    #[default]
    Person,
    /// Software acting on its own behalf.
    Agent,
    /// A person and a model working together, either driving.
    Assisted,
}

impl AgentKind {
    /// Stable wire name. Changing one is a breaking protocol change.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Agent => "agent",
            Self::Assisted => "assisted",
        }
    }

    /// Parse a wire name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "person" => Some(Self::Person),
            "agent" => Some(Self::Agent),
            "assisted" => Some(Self::Assisted),
            _ => None,
        }
    }

    /// Whether software is involved at all.
    #[must_use]
    pub const fn is_automated(self) -> bool {
        matches!(self, Self::Agent | Self::Assisted)
    }

    /// Byte tag for canonical encoding.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Person => 0,
            Self::Agent => 1,
            Self::Assisted => 2,
        }
    }

    /// Decode a byte tag.
    #[must_use]
    pub const fn from_u8(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Person),
            1 => Some(Self::Agent),
            2 => Some(Self::Assisted),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AgentKind;

    #[test]
    fn every_kind_round_trips_both_ways() {
        for kind in [AgentKind::Person, AgentKind::Agent, AgentKind::Assisted] {
            assert_eq!(AgentKind::from_name(kind.as_str()), Some(kind));
            assert_eq!(AgentKind::from_u8(kind.as_u8()), Some(kind));
        }
        assert_eq!(AgentKind::from_name("robot"), None);
        assert_eq!(AgentKind::from_u8(7), None);
    }

    #[test]
    fn declaring_yourself_software_is_disclosure_not_a_class() {
        // The type carries no capability and no handicap: there is nothing on
        // it to grant or withhold. Any rule that treated an agent differently
        // would have to be written somewhere else, deliberately, and reviewed.
        assert!(AgentKind::Agent.is_automated());
        assert!(AgentKind::Assisted.is_automated());
        assert!(!AgentKind::Person.is_automated());
        assert_eq!(AgentKind::default(), AgentKind::Person);
    }
}
