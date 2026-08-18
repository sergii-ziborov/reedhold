//! Shared identifiers, domain tags, and errors for Reedhold.

#![forbid(unsafe_code)]

mod domain;
mod error;
mod ids;
mod network;

pub use domain::{DomainTag, PROTOCOL_NAME, PROTOCOL_VERSION};
pub use error::{Error, Result};
pub use ids::{ContentId, DeviceId, Digest32, IdentityId};
pub use network::NetworkId;
