//! Proof of contribution. Credits move. History does not. Popularity is not consensus.

#![forbid(unsafe_code)]

mod book;
mod kind;
mod math;
mod mint;
mod score;

pub use book::Book;
pub use kind::WorkKind;
pub use mint::{EPOCH_MINT_BUDGET, TARGET_EPOCH_WORK, epoch_budget, minted_total, settle};
pub use score::Score;
