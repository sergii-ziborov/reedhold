//! How a datagram left the sender this epoch.

use crate::ports::PeerId;

/// Observed send path. Never requires the company host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Route {
    /// Recipient was online.
    Direct,
    /// Parked at a transitional relay until the recipient appears.
    ViaRelay(PeerId),
    /// Carried closer to the target through neighbours, hop by hop.
    Hops(Vec<PeerId>),
    /// The next hop lives in another process; the host must post it there.
    Remote(String, PeerId),
    /// No live hop; sender keeps it until someone is reachable.
    HeldLocal,
}

impl Route {
    /// Stable name for the host API.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::ViaRelay(_) => "relay",
            Self::Hops(_) => "hops",
            Self::Remote(_, _) => "remote",
            Self::HeldLocal => "held",
        }
    }

    /// How many peers touched the payload before it landed.
    #[must_use]
    pub fn hop_count(&self) -> usize {
        match self {
            Self::Direct | Self::HeldLocal => 0,
            Self::ViaRelay(_) | Self::Remote(_, _) => 1,
            Self::Hops(path) => path.len(),
        }
    }
}
