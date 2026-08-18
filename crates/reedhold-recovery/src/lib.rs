//! Password vault and signed recovery manifest.

#![forbid(unsafe_code)]

mod field;
mod manifest;
mod params;
mod shares;
mod vault;

pub use manifest::RecoveryManifest;
pub use params::KdfParams;
pub use shares::{SeedShare, combine_seed, split_seed};
pub use vault::{SealedSeed, seal_seed, unseal_seed};
