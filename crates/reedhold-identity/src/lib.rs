//! Recoverable identity: seed, roots, and authorized devices.

#![forbid(unsafe_code)]

mod derive;
mod device;
mod seed;

pub use device::{DeviceAuthority, DeviceKeys, verify_device};
pub use seed::{IdentityBundle, MasterSeed};
