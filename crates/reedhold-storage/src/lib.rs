//! Durability classes, erasure, placement, and a bounded shard grid.

#![forbid(unsafe_code)]

mod budget;
mod code;
mod grid;
mod holder;
mod place;
mod tier;

pub use budget::NodeBudget;
pub use code::{Coding, Shard};
pub use grid::{Grid, ObjectMeta};
pub use holder::HolderId;
pub use tier::DurabilityTier;
