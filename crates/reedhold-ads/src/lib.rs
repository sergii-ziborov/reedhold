//! Advertising capability. Not a network master key.

#![forbid(unsafe_code)]

mod auction;
mod bucket;
mod certificate;
mod creative;
mod distributor;
mod limits;
mod market;
mod math;
mod root;

pub use auction::{Bid, Clearing};
pub use bucket::{DISTRIBUTOR_MIN_BUCKET, bucket, floor};
pub use certificate::AdOperatorCertificate;
pub use creative::Creative;
pub use distributor::Distributor;
pub use limits::AdvertisingLimits;
pub use market::{Market, Split};
pub use root::AdvertisingRoot;
