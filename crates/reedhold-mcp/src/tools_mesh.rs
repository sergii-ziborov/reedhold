//! MCP tools for the in-process mesh fabric.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value};
use reedhold_core::Error;

pub(crate) fn mesh_open(host: &mut Host, arguments: Value) -> ToolReply {
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
    match host.mesh_open(epoch, &prior, &candidates, company) {
        Ok(()) => ToolReply::structured("mesh-open"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn mesh_online(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.mesh_online(peer), "online")
}

pub(crate) fn mesh_offline(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.mesh_offline(peer), "offline")
}

pub(crate) fn mesh_block(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.mesh_block(peer), "blocked")
}

pub(crate) fn mesh_send(host: &mut Host, arguments: Value) -> ToolReply {
    let from = match required_str(&arguments, "from") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let to = match required_str(&arguments, "to") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let payload = match required_str(&arguments, "payload") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.mesh_send(from, to, payload) {
        Ok(route) => ToolReply::structured(route),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn mesh_drain(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "peer") {
        Ok(peer) => match host.mesh_drain(peer) {
            Ok(messages) => ToolReply::structured(messages),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn peer_op(
    arguments: Value,
    op: impl FnOnce(&str) -> reedhold_core::Result<()>,
    ok: &str,
) -> ToolReply {
    match required_str(&arguments, "peer") {
        Ok(peer) => match op(peer) {
            Ok(()) => ToolReply::structured(ok),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, Error> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Codec("missing string field"))
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
        out.push(
            value
                .as_str()
                .ok_or(Error::Codec("array entry must be a string"))?
                .to_owned(),
        );
    }
    Ok(out)
}
