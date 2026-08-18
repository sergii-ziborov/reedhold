//! Recoverable identity: seed, roots, and authorized devices.

#![forbid(unsafe_code)]

mod derive;
mod device;
mod grant;
mod root;
mod seed;

pub use device::{DeviceAuthority, DeviceKeys, verify_device};
pub use grant::DeviceGrant;
pub use root::IdentityRoot;
pub use seed::{IdentityBundle, MasterSeed};
