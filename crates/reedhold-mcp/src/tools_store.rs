//! Store and threshold-recovery MCP tools.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value};
use reedhold_api::ShareView;
use reedhold_core::Error;

pub(crate) fn save_store(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "path") {
        Ok(path) => match host.save(path) {
            Ok(()) => ToolReply::structured("saved"),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn load_store(host: &mut Host, arguments: Value) -> ToolReply {
    let path = match required_str(&arguments, "path") {
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
    match host.load(path, password, device) {
        Ok(account) => ToolReply::structured(account),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn split_recovery(host: &mut Host, arguments: Value) -> ToolReply {
    let threshold = match required_str(&arguments, "threshold").and_then(parse_u8) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let total = match required_str(&arguments, "total").and_then(parse_u8) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.split_recovery(threshold, total) {
        Ok(shares) => ToolReply::structured(shares),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn combine_recovery(host: &mut Host, arguments: Value) -> ToolReply {
    let threshold = match required_str(&arguments, "threshold").and_then(parse_u8) {
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
    let shares = match parse_shares(&arguments) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.combine_recovery(&shares, threshold, password, device) {
        Ok((account, manifest)) => ToolReply::structured(mcport::json!({
            "account": account,
            "manifest": manifest,
        })),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn parse_shares(arguments: &Value) -> Result<Vec<ShareView>, Error> {
    let array = arguments
        .get("shares")
        .and_then(Value::as_array)
        .ok_or(Error::Recovery("missing shares"))?;
    let mut shares = Vec::with_capacity(array.len());
    for value in array {
        let index = value
            .get("index")
            .and_then(Value::as_str)
            .ok_or(Error::Recovery("share index must be a string"))?
            .parse::<u8>()
            .map_err(|_| Error::Recovery("bad share index"))?;
        let body_hex = value
            .get("body_hex")
            .and_then(Value::as_str)
            .ok_or(Error::Recovery("share body_hex missing"))?
            .to_owned();
        shares.push(ShareView { index, body_hex });
    }
    Ok(shares)
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, Error> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Codec("missing string field"))
}

fn parse_u8(text: &str) -> Result<u8, Error> {
    text.parse()
        .map_err(|_| Error::Codec("expected a small integer"))
}
