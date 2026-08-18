//! Signed social events.

#![forbid(unsafe_code)]

mod kind;
mod signed;

pub use kind::EventKind;
pub use signed::{SignedEvent, content_id, sign_event};
