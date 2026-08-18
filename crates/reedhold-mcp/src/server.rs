//! `mcport` server composition.

use crate::host::Host;
use crate::tools;
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
