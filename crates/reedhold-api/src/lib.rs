//! Host-facing sync session. No live transport, no MCP, no async runtime.

#![forbid(unsafe_code)]

mod ads;
mod alias;
mod chain;
mod contacts;
mod durable;
mod inbox;
mod market;
mod mesh;
mod persist;
mod postbox;
mod privacy;
mod rep;
mod rooms;
mod session;
mod shares;
mod sync;
mod talk;
mod view;
mod work;

pub use ads::{AdvertisingLimitsView, advertising_limits};
pub use alias::{AliasDirectory, AliasView};
pub use chain::{ChainSession, HeaderView, ProofView};
pub use contacts::{ContactView, RequestView};
pub use durable::{DurableSession, ObjectView};
pub use inbox::{CircleView, TalkView};
pub use market::{ClearingView, MarketSession, SplitView};
pub use mesh::{MeshSession, RouteView};
pub use privacy::{MessagePolicy, PrivacyView};
pub use rep::{ContentScoreView, IdentityScoreView, ReactionView, RepSession};
pub use rooms::{RoomBoard, RoomPostView, RoomView, TOPIC_CATALOG};
pub use session::{Created, Session};
pub use shares::{ShareView, session_from_shares};
pub use sync::{SyncPlanView, sync_plan};
pub use talk::{TalkNet, dm_conversation_hex};
pub use view::{AccountView, EventView, ManifestView, invariants};
pub use work::{WorkSession, WorkView};
