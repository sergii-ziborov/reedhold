//! Advertising-market tools in the MCP catalog.

use crate::host::Host;
use crate::schema::{ads_bid_schema, ads_book_schema, ads_post_schema, object_schema};
use crate::tools_ads;
use mcport::McpServer;

pub(crate) fn with_ads(server: McpServer<Host>) -> McpServer<Host> {
    server
        .tool_with_state(
            "ads_open",
            "Open the attention market. Genesis is not required.",
            object_schema(&[]),
            tools_ads::ads_open,
        )
        .tool_with_state(
            "ads_post",
            "Post a campaign. Payload is a content id, never ad bytes or a user id.",
            ads_post_schema(),
            tools_ads::ads_post,
        )
        .tool_with_state(
            "ads_register",
            "Register a distributor by strength. Weak accounts are rejected.",
            object_schema(&["id", "strength"]),
            tools_ads::ads_register,
        )
        .tool_with_state(
            "ads_bid",
            "Sealed bid for one topic/bucket/epoch book.",
            ads_bid_schema(),
            tools_ads::ads_bid,
        )
        .tool_with_state(
            "ads_clear",
            "Batch uniform-price clearing. Runs without a genesis operator.",
            ads_book_schema(),
            tools_ads::ads_clear,
        )
        .tool_with_state(
            "ads_select",
            "Local selector: topic + bucket only. There is no user-id targeting API.",
            ads_book_schema(),
            tools_ads::ads_select,
        )
        .tool_with_state(
            "ads_hide",
            "Hide/dislike a campaign. Raises its future floor.",
            object_schema(&["campaign"]),
            tools_ads::ads_hide,
        )
        .tool_with_state(
            "ads_settle",
            "Split sandbox credits. Not real money.",
            ads_book_schema(),
            tools_ads::ads_settle,
        )
}
