//! Advertising capability. Not a network master key.

#![forbid(unsafe_code)]

mod certificate;
mod limits;
mod root;

pub use certificate::AdOperatorCertificate;
pub use limits::AdvertisingLimits;
pub use root::AdvertisingRoot;
