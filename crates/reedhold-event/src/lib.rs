//! Signed social events.

#![forbid(unsafe_code)]

mod envelope;
mod kind;
mod signed;
mod talk;

pub use envelope::{MessageEnvelope, open_message, seal_message};
pub use kind::EventKind;
pub use signed::{SignedEvent, content_id, sign_event};
pub use talk::{InviteBody, TalkBody, TalkPacket, dm_conversation};
