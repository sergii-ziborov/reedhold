//! Account lifecycle over identity, recovery, and events.

#![forbid(unsafe_code)]

mod account;
mod restore;

pub use account::{Account, CreatedAccount, create_account};
pub use restore::restore_account;
