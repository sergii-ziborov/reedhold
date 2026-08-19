//! MCP tools for proof of contribution.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value, json};
use reedhold_core::Error;

pub(crate) fn work_open(host: &mut Host, _arguments: Value) -> ToolReply {
    host.work_open();
    ToolReply::structured("work-open")
}

pub(crate) fn work_record(host: &mut Host, arguments: Value) -> ToolReply {
    let node = match required_str(&arguments, "node") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let kind = match required_str(&arguments, "kind") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let units = match required_str(&arguments, "units").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let epoch = match required_str(&arguments, "epoch").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let reliable = required_str(&arguments, "reliable").unwrap_or("1") != "0";
    match host.work_record(node, kind, units, epoch, reliable) {
        Ok(minted) => ToolReply::structured(json!({ "minted": minted })),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn work_view(host: &mut Host, arguments: Value) -> ToolReply {
    let node = match required_str(&arguments, "node") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let social = match required_str(&arguments, "social").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.work_view(node, social) {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn work_transfer(host: &mut Host, arguments: Value) -> ToolReply {
    let from = match required_str(&arguments, "from") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let to = match required_str(&arguments, "to") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let amount = match required_str(&arguments, "amount").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.work_transfer(from, to, amount) {
        Ok(()) => ToolReply::structured("transferred"),
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
