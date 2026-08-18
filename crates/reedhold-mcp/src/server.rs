//! `mcport` server composition.

use crate::host::Host;
use crate::tools;
use crate::tools_store;
use mcport::{McpServer, Value, json};

/// Build the Reedhold MCP server.
#[must_use]
pub fn build_server() -> McpServer<Host> {
    McpServer::with_state("reedhold", env!("CARGO_PKG_VERSION"), Host::default())
        .instructions(
            "Reedhold protocol session. Create or restore an account, then emit or verify signed events. No company server is required.",
        )
        .strict_schemas()
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
}

fn object_schema(required: &[&str]) -> Value {
    let mut properties = mcport::Map::new();
    for field in required {
        properties.insert((*field).into(), json!({ "type": "string" }));
    }
    let required = required
        .iter()
        .map(|field| Value::String((*field).to_owned()))
        .collect();
    json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": Value::Array(required),
        "additionalProperties": false
    })
}

fn sync_plan_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "epoch": { "type": "string" },
            "prior_commit": { "type": "string" },
            "candidates": { "type": "array", "items": { "type": "string" } },
            "company": { "type": "string" }
        },
        "required": ["epoch", "candidates"],
        "additionalProperties": false
    })
}

fn combine_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "threshold": { "type": "string" },
            "password": { "type": "string" },
            "device_secret": { "type": "string" },
            "shares": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "index": { "type": "string" },
                        "body_hex": { "type": "string" }
                    },
                    "required": ["index", "body_hex"]
                }
            }
        },
        "required": ["threshold", "password", "device_secret", "shares"],
        "additionalProperties": false
    })
}
