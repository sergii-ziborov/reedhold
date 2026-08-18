//! Host-facing sync session. No live transport, no MCP, no async runtime.

#![forbid(unsafe_code)]

mod ads;
mod durable;
mod inbox;
mod mesh;
mod persist;
mod session;
mod shares;
mod sync;
mod talk;
mod view;

pub use ads::{AdvertisingLimitsView, advertising_limits};
pub use durable::{DurableSession, ObjectView};
pub use inbox::{CircleView, TalkView};
pub use mesh::{MeshSession, RouteView};
pub use session::{Created, Session};
pub use shares::{ShareView, session_from_shares};
pub use sync::{SyncPlanView, sync_plan};
pub use talk::TalkNet;
pub use view::{AccountView, EventView, ManifestView, invariants};
