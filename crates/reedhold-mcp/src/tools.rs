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

pub(crate) fn sync_plan(_host: &mut Host, arguments: Value) -> ToolReply {
    let epoch = match required_str(&arguments, "epoch").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let prior = required_str(&arguments, "prior_commit").unwrap_or("");
    let prior = if prior.is_empty() {
        "00".repeat(32)
    } else {
        prior.to_owned()
    };
    let candidates = match string_list(&arguments, "candidates") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let company = arguments.get("company").and_then(Value::as_str);
    match reedhold_api::sync_plan(epoch, &prior, &candidates, company, None) {
        Ok(plan) => ToolReply::structured(plan),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn advertising_limits(_host: &mut Host, _arguments: Value) -> ToolReply {
    ToolReply::structured(reedhold_api::advertising_limits())
}

fn parse_u64(text: &str) -> Result<u64, Error> {
    text.parse()
        .map_err(|_| Error::Codec("expected an unsigned integer"))
}

fn string_list(arguments: &Value, key: &str) -> Result<Vec<String>, Error> {
    let array = arguments
        .get(key)
        .and_then(Value::as_array)
        .ok_or(Error::Codec("missing string array"))?;
    let mut out = Vec::with_capacity(array.len());
    for value in array {
        let text = value
            .as_str()
            .ok_or(Error::Codec("array entry must be a string"))?;
        out.push(text.to_owned());
    }
    Ok(out)
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, Error> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Codec("missing string field"))
}
