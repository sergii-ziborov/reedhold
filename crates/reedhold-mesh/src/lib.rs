//! Mesh ports. Peer discovery is not consensus.

#![forbid(unsafe_code)]

mod ports;

pub use ports::{DiscoveryHint, PeerId, TransportKind};
