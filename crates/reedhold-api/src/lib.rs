//! Host-facing sync session. No live transport, no MCP, no async runtime.

#![forbid(unsafe_code)]

mod ads;
mod session;
mod sync;
mod view;

pub use ads::{AdvertisingLimitsView, advertising_limits};
pub use session::{Created, Session};
pub use sync::{SyncPlanView, sync_plan};
pub use view::{AccountView, EventView, ManifestView, invariants};
