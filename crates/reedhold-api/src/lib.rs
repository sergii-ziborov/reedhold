//! Host-facing sync session. No live transport, no MCP, no async runtime.

#![forbid(unsafe_code)]

mod ads;
mod chain;
mod durable;
mod inbox;
mod market;
mod mesh;
mod persist;
mod rep;
mod session;
mod shares;
mod sync;
mod talk;
mod view;
mod work;

pub use ads::{AdvertisingLimitsView, advertising_limits};
pub use chain::{ChainSession, HeaderView, ProofView};
pub use durable::{DurableSession, ObjectView};
pub use inbox::{CircleView, TalkView};
pub use market::{ClearingView, MarketSession, SplitView};
pub use mesh::{MeshSession, RouteView};
pub use rep::{ContentScoreView, IdentityScoreView, ReactionView, RepSession};
pub use session::{Created, Session};
pub use shares::{ShareView, session_from_shares};
pub use sync::{SyncPlanView, sync_plan};
pub use talk::TalkNet;
pub use view::{AccountView, EventView, ManifestView, invariants};
pub use work::{WorkSession, WorkView};
