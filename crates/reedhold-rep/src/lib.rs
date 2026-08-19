//! Reputation v0. Reactions mature. Reputation is not a token.

#![forbid(unsafe_code)]

mod factor;
mod graph;
mod identity;
mod kind;
mod maturity;
mod milli;
mod reaction;
mod transfer;

pub use factor::epoch_budget;
pub use graph::{ContentScore, Graph};
pub use identity::IdentityRep;
pub use kind::ReactionKind;
pub use milli::Milli;
pub use reaction::Reaction;
pub use transfer::transfer;
