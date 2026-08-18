//! Host-facing sync session. No mesh, no MCP, no async runtime.

#![forbid(unsafe_code)]

mod session;
mod view;

pub use session::{Created, Session};
pub use view::{AccountView, EventView, ManifestView, invariants};
