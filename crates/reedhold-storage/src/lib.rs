//! Durability classes and node budgets.

#![forbid(unsafe_code)]

mod budget;
mod tier;

pub use budget::NodeBudget;
pub use tier::DurabilityTier;
