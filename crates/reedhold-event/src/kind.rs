//! Event kinds. New kinds are additive; numbers are forever.

/// Canonical social-event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum EventKind {
    /// Public or private post.
    Post = 1,
    /// Reply to another event.
    Reply = 2,
    /// Repost / amplify.
    Repost = 3,
    /// Like.
    Like = 4,
    /// Dislike.
    Dislike = 5,
    /// Stronger than a like. Spends influence budget later.
    Endorse = 6,
    /// Follow.
    Follow = 7,
    /// Unfollow.
    Unfollow = 8,
    /// Profile field update.
    ProfileUpdate = 9,
    /// Direct message envelope.
    DirectMessage = 10,
    /// Device authorization / revocation record.
    DeviceAuthorize = 11,
    /// Create a small group (shared epoch key, not MLS).
    GroupCreate = 12,
    /// Wrap a group epoch key for one member.
    GroupInvite = 13,
    /// Sealed small-group message.
    GroupMessage = 14,
    /// Member left or was removed. Key rotation is a later epoch.
    GroupLeave = 15,
}

impl EventKind {
    /// Wire number.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Parse a wire number.
    #[must_use]
    pub const fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::Post),
            2 => Some(Self::Reply),
            3 => Some(Self::Repost),
            4 => Some(Self::Like),
            5 => Some(Self::Dislike),
            6 => Some(Self::Endorse),
            7 => Some(Self::Follow),
            8 => Some(Self::Unfollow),
            9 => Some(Self::ProfileUpdate),
            10 => Some(Self::DirectMessage),
            11 => Some(Self::DeviceAuthorize),
            12 => Some(Self::GroupCreate),
            13 => Some(Self::GroupInvite),
            14 => Some(Self::GroupMessage),
            15 => Some(Self::GroupLeave),
            _ => None,
        }
    }

    /// Stable name for the host API and MCP.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Post => "post",
            Self::Reply => "reply",
            Self::Repost => "repost",
            Self::Like => "like",
            Self::Dislike => "dislike",
            Self::Endorse => "endorse",
            Self::Follow => "follow",
            Self::Unfollow => "unfollow",
            Self::ProfileUpdate => "profile_update",
            Self::DirectMessage => "direct_message",
            Self::DeviceAuthorize => "device_authorize",
            Self::GroupCreate => "group_create",
            Self::GroupInvite => "group_invite",
            Self::GroupMessage => "group_message",
            Self::GroupLeave => "group_leave",
        }
    }

    /// Parse a host-API name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "post" => Some(Self::Post),
            "reply" => Some(Self::Reply),
            "repost" => Some(Self::Repost),
            "like" => Some(Self::Like),
            "dislike" => Some(Self::Dislike),
            "endorse" => Some(Self::Endorse),
            "follow" => Some(Self::Follow),
            "unfollow" => Some(Self::Unfollow),
            "profile_update" => Some(Self::ProfileUpdate),
            "direct_message" => Some(Self::DirectMessage),
            "device_authorize" => Some(Self::DeviceAuthorize),
            "group_create" => Some(Self::GroupCreate),
            "group_invite" => Some(Self::GroupInvite),
            "group_message" => Some(Self::GroupMessage),
            "group_leave" => Some(Self::GroupLeave),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventKind;

    #[test]
    fn numbers_round_trip() {
        for kind in [
            EventKind::Post,
            EventKind::Reply,
            EventKind::DirectMessage,
            EventKind::Endorse,
        ] {
            assert_eq!(EventKind::from_u16(kind.as_u16()), Some(kind));
        }
        assert_eq!(EventKind::from_u16(0), None);
    }
}
