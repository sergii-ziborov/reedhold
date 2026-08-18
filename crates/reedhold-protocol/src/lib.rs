//! Account lifecycle over identity, recovery, and events.

#![forbid(unsafe_code)]

mod account;
mod circle;
mod restore;

pub use account::{Account, CreatedAccount, create_account, open_seed};
pub use circle::Circle;
pub use restore::restore_account;
