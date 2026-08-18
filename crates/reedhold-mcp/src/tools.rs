//! MCP tool handlers. Results serialize through `blazingly-json` via mcport.

#![allow(clippy::needless_pass_by_value)] // `mcport` handlers take owned `Value`.

use crate::host::Host;
use mcport::{ToolReply, Value, json};
use reedhold_api::invariants;
use reedhold_core::Error;

pub(crate) fn create_account(host: &mut Host, arguments: Value) -> ToolReply {
    let password = match required_str(&arguments, "password") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let device = match required_str(&arguments, "device_secret") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.create(password, device) {
        Ok((account, manifest)) => ToolReply::structured(json!({
            "account": account,
            "manifest": manifest,
        })),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn restore_account(host: &mut Host, arguments: Value) -> ToolReply {
    let manifest = match required_str(&arguments, "manifest_hex") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let password = match required_str(&arguments, "password") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let device = match required_str(&arguments, "device_secret") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.restore(manifest, password, device) {
        Ok(account) => ToolReply::structured(account),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn account(host: &mut Host, _arguments: Value) -> ToolReply {
    match host.view() {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn emit(host: &mut Host, arguments: Value) -> ToolReply {
    let kind = match required_str(&arguments, "kind") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let payload = match required_str(&arguments, "payload") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.emit(kind, payload) {
        Ok(event) => ToolReply::structured(event),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn verify(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "event_hex") {
        Ok(event) => match host.verify(event) {
            Ok(view) => ToolReply::structured(view),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn change_password(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "password") {
        Ok(password) => match host.change_password(password) {
            Ok(manifest) => ToolReply::structured(manifest),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn emit_sealed(host: &mut Host, arguments: Value) -> ToolReply {
    let key = match required_str(&arguments, "conversation_key") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let plain = match required_str(&arguments, "plaintext") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.emit_sealed(key, plain) {
        Ok(event) => ToolReply::structured(event),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn list_invariants(_host: &mut Host, _arguments: Value) -> ToolReply {
    ToolReply::structured(invariants())
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, Error> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Codec("missing string field"))
}
