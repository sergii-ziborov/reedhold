//! How a datagram left the sender this epoch.

use crate::ports::PeerId;

/// Observed send path. Never requires the company host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    /// Recipient was online.
    Direct,
    /// Parked at a transitional relay until the recipient appears.
    ViaRelay(PeerId),
    /// No live hop; sender keeps it until someone is reachable.
    HeldLocal,
}

impl Route {
    /// Stable name for the host API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::ViaRelay(_) => "relay",
            Self::HeldLocal => "held",
        }
    }
}
