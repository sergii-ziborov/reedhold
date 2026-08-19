//! MCP tools for compact checkpoints.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value, json};
use reedhold_core::Error;

pub(crate) fn chain_open(host: &mut Host, _arguments: Value) -> ToolReply {
    match host.chain_open() {
        Ok(()) => ToolReply::structured("chain-open"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn chain_commit(host: &mut Host, arguments: Value) -> ToolReply {
    let epoch = match required_str(&arguments, "epoch").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let identity = required_str(&arguments, "identity").unwrap_or("");
    let groups = required_str(&arguments, "groups").unwrap_or("");
    let storage = required_str(&arguments, "storage").unwrap_or("");
    match host.chain_commit(epoch, identity, groups, storage) {
        Ok(header) => ToolReply::structured(header),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn chain_head(host: &mut Host, _arguments: Value) -> ToolReply {
    match host.chain_head() {
        Ok(header) => ToolReply::structured(header),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn chain_headers(host: &mut Host, _arguments: Value) -> ToolReply {
    match host.chain_headers() {
        Ok(headers) => ToolReply::structured(headers),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn chain_prove(host: &mut Host, arguments: Value) -> ToolReply {
    let leaves = match string_list(&arguments, "leaves") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let index = match required_str(&arguments, "index").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.chain_prove(&leaves, index) {
        Ok(proof) => ToolReply::structured(proof),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn chain_verify(host: &mut Host, arguments: Value) -> ToolReply {
    let leaf = match required_str(&arguments, "leaf") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let root = match required_str(&arguments, "root") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let index = match required_str(&arguments, "index").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let siblings = match string_list(&arguments, "siblings") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.chain_verify(leaf, root, index, &siblings) {
        Ok(ok) => ToolReply::structured(json!({ "ok": ok })),
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

fn parse_u32(text: &str) -> Result<u32, Error> {
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
