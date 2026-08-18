//! Mesh ports. Peer discovery is not consensus.

#![forbid(unsafe_code)]

mod epoch;
mod lottery;
mod plan;
mod ports;

pub use epoch::{EPOCH_SECONDS, EpochSeed, SyncEpoch};
pub use lottery::{relay_score, select_relays};
pub use plan::{DEFAULT_RELAY_COUNT, HostRole, SyncPlan};
pub use ports::{DiscoveryHint, PeerId, TransportKind};
