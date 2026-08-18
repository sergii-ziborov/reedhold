//! Split MCP catalog so no function exceeds the 100-line budget.

use crate::host::Host;
use crate::schema::{combine_schema, holders_schema, object_schema, sync_plan_schema};
use crate::tools;
use crate::tools_durable;
use crate::tools_mesh;
use crate::tools_store;
use mcport::McpServer;

pub(crate) fn with_account_tools(server: McpServer<Host>) -> McpServer<Host> {
    server
        .tool_with_state(
            "create_account",
            "Create a recoverable identity. Password unlocks the vault; it is not the identity.",
            object_schema(&["password", "device_secret"]),
            tools::create_account,
        )
        .tool_with_state(
            "restore_account",
            "Restore an identity from a stored recovery manifest.",
            object_schema(&["manifest_hex", "password", "device_secret"]),
            tools::restore_account,
        )
        .tool_with_state(
            "account",
            "Show the unlocked account snapshot.",
            object_schema(&[]),
            tools::account,
        )
        .tool_with_state(
            "emit",
            "Sign a social event from the unlocked account.",
            object_schema(&["kind", "payload"]),
            tools::emit,
        )
        .tool_with_state(
            "verify",
            "Verify a signed event hex against the unlocked device key.",
            object_schema(&["event_hex"]),
            tools::verify,
        )
        .tool_with_state(
            "change_password",
            "Re-seal the same MasterSeed under a new password.",
            object_schema(&["password"]),
            tools::change_password,
        )
        .tool_with_state(
            "emit_sealed",
            "Seal a direct message and sign the envelope.",
            object_schema(&["conversation_key", "plaintext"]),
            tools::emit_sealed,
        )
        .tool_with_state(
            "list_invariants",
            "Protocol invariants that later stages must not drop.",
            object_schema(&[]),
            tools::list_invariants,
        )
        .tool_with_state(
            "sync_plan",
            "Draw this epoch's random transitional relays. Company host is never required.",
            sync_plan_schema(),
            tools::sync_plan,
        )
        .tool_with_state(
            "advertising_limits",
            "Genesis advertising token: market rights only, not network control.",
            object_schema(&[]),
            tools::advertising_limits,
        )
}

pub(crate) fn with_store_and_mesh(server: McpServer<Host>) -> McpServer<Host> {
    server
        .tool_with_state(
            "save_store",
            "Write the sealed manifest and signed event log to a local directory.",
            object_schema(&["path"]),
            tools_store::save_store,
        )
        .tool_with_state(
            "load_store",
            "Reinstall from a local directory using username-equivalent password.",
            object_schema(&["path", "password", "device_secret"]),
            tools_store::load_store,
        )
        .tool_with_state(
            "split_recovery",
            "Split the unlocked MasterSeed into k-of-n shares. One share is useless.",
            object_schema(&["threshold", "total"]),
            tools_store::split_recovery,
        )
        .tool_with_state(
            "combine_recovery",
            "Restore an identity from enough shares and a new password.",
            combine_schema(),
            tools_store::combine_recovery,
        )
        .tool_with_state(
            "mesh_open",
            "Open an in-process mesh fabric for this epoch's lottery.",
            sync_plan_schema(),
            tools_mesh::mesh_open,
        )
        .tool_with_state(
            "mesh_online",
            "Bring a mesh peer online and deliver waiting mail.",
            object_schema(&["peer"]),
            tools_mesh::mesh_online,
        )
        .tool_with_state(
            "mesh_offline",
            "Take a mesh peer offline. Relays keep its mail.",
            object_schema(&["peer"]),
            tools_mesh::mesh_offline,
        )
        .tool_with_state(
            "mesh_block",
            "Block a host. The fabric keeps running.",
            object_schema(&["peer"]),
            tools_mesh::mesh_block,
        )
        .tool_with_state(
            "mesh_send",
            "Send an opaque payload. Uses a rotating relay if the dest is offline.",
            object_schema(&["from", "to", "payload"]),
            tools_mesh::mesh_send,
        )
        .tool_with_state(
            "mesh_drain",
            "Read delivered payloads for a peer.",
            object_schema(&["peer"]),
            tools_mesh::mesh_drain,
        )
        .tool_with_state(
            "durable_open",
            "Open a durable shard grid. Company is never a required holder.",
            holders_schema(),
            tools_durable::durable_open,
        )
        .tool_with_state(
            "durable_put",
            "Erasure-encode and place an object (critical = 4-of-6).",
            object_schema(&["payload"]),
            tools_durable::durable_put,
        )
        .tool_with_state(
            "durable_get",
            "Reconstruct an object from any k live shards.",
            object_schema(&["id"]),
            tools_durable::durable_get,
        )
        .tool_with_state(
            "durable_kill",
            "Drop a holder and its contracted shards.",
            object_schema(&["holder"]),
            tools_durable::durable_kill,
        )
        .tool_with_state(
            "durable_repair",
            "Rebuild missing shards onto surviving holders.",
            object_schema(&["id"]),
            tools_durable::durable_repair,
        )
}
