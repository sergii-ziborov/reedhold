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
pub use vault::{SealedSeed, seal_seed, seal_seed_with, unseal_seed, unseal_seed_with};
