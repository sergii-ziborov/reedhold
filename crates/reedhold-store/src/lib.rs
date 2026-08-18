//! Local persistence. No company server, no plaintext seed.

#![forbid(unsafe_code)]

mod eventlog;
mod layout;

pub use eventlog::StoredEvent;
pub use layout::LocalStore;
