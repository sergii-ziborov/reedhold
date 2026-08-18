//! Mesh ports, daily lottery, and an in-process routing fabric.

#![forbid(unsafe_code)]

mod epoch;
mod fabric;
mod frame;
mod lottery;
mod node;
mod plan;
mod ports;
mod route;

pub use epoch::{EPOCH_SECONDS, EpochSeed, SyncEpoch};
pub use fabric::Fabric;
pub use frame::{FrameKind, MeshFrame};
pub use lottery::{relay_score, select_relays};
pub use plan::{DEFAULT_RELAY_COUNT, HostRole, SyncPlan};
pub use ports::{DiscoveryHint, PeerId, TransportKind};
pub use route::Route;
