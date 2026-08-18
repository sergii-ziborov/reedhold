//! Reedhold protocol facade.
//!
//! Layered crates stay independent. This crate only re-exports them.

#![forbid(unsafe_code)]
#![doc = include_str!("../../../README.md")]

pub use reedhold_ads as ads;
pub use reedhold_api as api;
pub use reedhold_chain as chain;
pub use reedhold_client as client;
pub use reedhold_codec as codec;
pub use reedhold_core as core;
pub use reedhold_event as event;
pub use reedhold_identity as identity;
pub use reedhold_mesh as mesh;
pub use reedhold_protocol as protocol;
pub use reedhold_recovery as recovery;
pub use reedhold_storage as storage;
pub use reedhold_store as store;

pub use reedhold_api::{AccountView, EventView, ManifestView, Session};
pub use reedhold_core::{
    ContentId, DeviceId, Digest32, DomainTag, Error, IdentityId, NetworkId, PROTOCOL_NAME,
    PROTOCOL_VERSION, Result,
};
pub use reedhold_event::{EventKind, SignedEvent, content_id, sign_event};
pub use reedhold_identity::{IdentityBundle, MasterSeed};
pub use reedhold_protocol::{Account, CreatedAccount, create_account, restore_account};
pub use reedhold_recovery::{KdfParams, RecoveryManifest};

/// Compiled crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
