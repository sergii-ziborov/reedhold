//! Shared identifiers, domain tags, and errors for Reedhold.

#![forbid(unsafe_code)]

mod domain;
mod error;
mod hex;
mod ids;
mod invariants;
mod network;

pub use domain::{DomainTag, PROTOCOL_NAME, PROTOCOL_VERSION};
pub use error::{Error, Result};
pub use hex::{decode as decode_hex, decode32, encode as encode_hex};
pub use ids::{ContentId, DeviceId, Digest32, IdentityId};
pub use invariants::INVARIANTS;
pub use network::NetworkId;
