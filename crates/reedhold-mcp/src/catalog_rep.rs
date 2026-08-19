//! Reputation tools in the MCP catalog.

use crate::host::Host;
use crate::schema::{object_schema, rep_react_schema, rep_seed_schema};
use crate::tools_rep;
use mcport::McpServer;

pub(crate) fn with_rep(server: McpServer<Host>) -> McpServer<Host> {
    server
        .tool_with_state(
            "rep_open",
            "Open a reputation book. Reactions mature; reputation is not a token.",
            object_schema(&[]),
            tools_rep::rep_open,
        )
        .tool_with_state(
            "rep_seed",
            "Seed identity dimensions for a simulation. Not a transfer.",
            rep_seed_schema(),
            tools_rep::rep_seed,
        )
        .tool_with_state(
            "rep_react",
            "Record a like, dislike, or endorse. Instant cluster pumps stay cheap.",
            rep_react_schema(),
            tools_rep::rep_react,
        )
        .tool_with_state(
            "rep_identity",
            "Show folded strength and remaining epoch influence budget.",
            object_schema(&["identity", "now"]),
            tools_rep::rep_identity,
        )
        .tool_with_state(
            "rep_content",
            "Show settled content reputation at host time `now`.",
            object_schema(&["target", "now"]),
            tools_rep::rep_content,
        )
        .tool_with_state(
            "rep_transfer",
            "Always fails. Reputation cannot be sent or sold.",
            object_schema(&["from", "to"]),
            tools_rep::rep_transfer,
        )
}
