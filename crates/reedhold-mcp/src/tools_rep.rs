//! MCP tools for reputation v0.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value};
use reedhold_core::Error;

pub(crate) fn rep_open(host: &mut Host, _arguments: Value) -> ToolReply {
    host.rep_open();
    ToolReply::structured("rep-open")
}

pub(crate) fn rep_seed(host: &mut Host, arguments: Value) -> ToolReply {
    let identity = match required_str(&arguments, "identity") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let continuity = match required_str(&arguments, "continuity").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let social = match required_str(&arguments, "social").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let content = match required_str(&arguments, "content").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let curation = match required_str(&arguments, "curation").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.rep_seed(identity, continuity, social, content, curation) {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn rep_react(host: &mut Host, arguments: Value) -> ToolReply {
    let author = match required_str(&arguments, "author") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let target = match required_str(&arguments, "target") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let kind = match required_str(&arguments, "kind") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let now = match required_str(&arguments, "now").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let cluster = required_str(&arguments, "cluster").unwrap_or("");
    match host.rep_react(author, target, kind, cluster, now) {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn rep_identity(host: &mut Host, arguments: Value) -> ToolReply {
    let identity = match required_str(&arguments, "identity") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let now = match required_str(&arguments, "now").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.rep_identity(identity, now) {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn rep_content(host: &mut Host, arguments: Value) -> ToolReply {
    let target = match required_str(&arguments, "target") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let now = match required_str(&arguments, "now").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.rep_content(target, now) {
        Ok(view) => ToolReply::structured(view),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn rep_transfer(_host: &mut Host, arguments: Value) -> ToolReply {
    let from = match required_str(&arguments, "from") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let to = match required_str(&arguments, "to") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match Host::rep_transfer(from, to, 1) {
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
