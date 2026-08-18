//! MCP tools for DMs and small groups.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value};
use reedhold_core::Error;

pub(crate) fn talk_open(host: &mut Host, arguments: Value) -> ToolReply {
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
    match host.talk_open(epoch, &prior, &candidates, company) {
        Ok(()) => ToolReply::structured("talk-open"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn talk_online(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.talk_online(peer), "online")
}

pub(crate) fn talk_offline(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.talk_offline(peer), "offline")
}

pub(crate) fn talk_block(host: &mut Host, arguments: Value) -> ToolReply {
    peer_op(arguments, |peer| host.talk_block(peer), "blocked")
}

pub(crate) fn talk_dm(host: &mut Host, arguments: Value) -> ToolReply {
    let to = match required_str(&arguments, "to") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let to_msg_pub = match required_str(&arguments, "to_msg_pub") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let plaintext = match required_str(&arguments, "plaintext") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.talk_dm(to, to_msg_pub, plaintext) {
        Ok(route) => ToolReply::structured(route),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn talk_create_group(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "name") {
        Ok(name) => match host.talk_create_group(name) {
            Ok(group) => ToolReply::structured(group),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn talk_invite(host: &mut Host, arguments: Value) -> ToolReply {
    let group = match required_str(&arguments, "group") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let member = match required_str(&arguments, "member") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let member_msg_pub = match required_str(&arguments, "member_msg_pub") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.talk_invite(group, member, member_msg_pub) {
        Ok(route) => ToolReply::structured(route),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn talk_send(host: &mut Host, arguments: Value) -> ToolReply {
    let group = match required_str(&arguments, "group") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let plaintext = match required_str(&arguments, "plaintext") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.talk_send(group, plaintext) {
        Ok(routes) => ToolReply::structured(routes),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn talk_inbox(host: &mut Host, _arguments: Value) -> ToolReply {
    match host.talk_inbox() {
        Ok(items) => ToolReply::structured(items),
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
