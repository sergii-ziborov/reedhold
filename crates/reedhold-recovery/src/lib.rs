//! Password vault and signed recovery manifest.

#![forbid(unsafe_code)]

mod manifest;
mod params;
mod vault;

pub use manifest::RecoveryManifest;
pub use params::KdfParams;
pub use vault::{SealedSeed, seal_seed, unseal_seed};
