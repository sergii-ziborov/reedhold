//! Proof-of-contribution tools in the MCP catalog.

use crate::host::Host;
use crate::schema::{object_schema, work_record_schema};
use crate::tools_work;
use mcport::McpServer;

pub(crate) fn with_work(server: McpServer<Host>) -> McpServer<Host> {
    server
        .tool_with_state(
            "work_open",
            "Open the contribution book. Credits move; history does not.",
            object_schema(&[]),
            tools_work::work_open,
        )
        .tool_with_state(
            "work_record",
            "Record storage/relay/repair/uptime work and mint sandbox credits.",
            work_record_schema(),
            tools_work::work_record,
        )
        .tool_with_state(
            "work_view",
            "Show weight, credits, and consensus eligibility. Popularity alone is never enough.",
            object_schema(&["node", "social"]),
            tools_work::work_view,
        )
        .tool_with_state(
            "work_transfer",
            "Move credits. Contribution history stays with the sender.",
            object_schema(&["from", "to", "amount"]),
            tools_work::work_transfer,
        )
}
