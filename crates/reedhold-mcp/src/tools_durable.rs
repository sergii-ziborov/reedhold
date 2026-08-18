//! MCP tools for the durable shard grid.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value};
use reedhold_core::Error;

pub(crate) fn durable_open(host: &mut Host, arguments: Value) -> ToolReply {
    let holders = match string_list(&arguments, "holders") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let company = arguments.get("company").and_then(Value::as_str);
    match host.durable_open(&holders, company) {
        Ok(()) => ToolReply::structured("durable-open"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn durable_put(host: &mut Host, arguments: Value) -> ToolReply {
    let payload = match required_str(&arguments, "payload") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let tier = arguments
        .get("tier")
        .and_then(Value::as_str)
        .unwrap_or("critical");
    match host.durable_put(payload, tier) {
        Ok(object) => ToolReply::structured(object),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn durable_get(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "id") {
        Ok(id) => match host.durable_get(id) {
            Ok(payload) => ToolReply::structured(payload),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn durable_kill(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "holder") {
        Ok(holder) => match host.durable_kill(holder) {
            Ok(()) => ToolReply::structured("killed"),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn durable_repair(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "id") {
        Ok(id) => match host.durable_repair(id) {
            Ok(object) => ToolReply::structured(object),
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
