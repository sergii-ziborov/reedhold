//! MCP tools for the advertising sandbox.

#![allow(clippy::needless_pass_by_value)]

use crate::host::Host;
use mcport::{ToolReply, Value, json};
use reedhold_core::Error;

pub(crate) fn ads_open(host: &mut Host, _arguments: Value) -> ToolReply {
    host.ads_open();
    ToolReply::structured("ads-open")
}

pub(crate) fn ads_post(host: &mut Host, arguments: Value) -> ToolReply {
    let advertiser = match required_str(&arguments, "advertiser") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let campaign = match required_str(&arguments, "campaign") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let payload = match required_str(&arguments, "payload") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let topic = match required_str(&arguments, "topic") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let bucket_min = match required_str(&arguments, "bucket_min").and_then(parse_u8) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let bucket_max = match required_str(&arguments, "bucket_max").and_then(parse_u8) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let budget = match required_str(&arguments, "budget").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let expiry = match required_str(&arguments, "expiry").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.ads_post(
        advertiser, campaign, payload, topic, bucket_min, bucket_max, budget, expiry,
    ) {
        Ok(()) => ToolReply::structured("posted"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_register(host: &mut Host, arguments: Value) -> ToolReply {
    let id = match required_str(&arguments, "id") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let strength = match required_str(&arguments, "strength").and_then(parse_u32) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.ads_register(id, strength) {
        Ok(()) => ToolReply::structured("registered"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_bid(host: &mut Host, arguments: Value) -> ToolReply {
    let advertiser = match required_str(&arguments, "advertiser") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let campaign = match required_str(&arguments, "campaign") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let topic = match required_str(&arguments, "topic") {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let bucket = match required_str(&arguments, "bucket").and_then(parse_u8) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let epoch = match required_str(&arguments, "epoch").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let price = match required_str(&arguments, "price").and_then(parse_u64) {
        Ok(value) => value,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    match host.ads_bid(advertiser, campaign, topic, bucket, epoch, price) {
        Ok(()) => ToolReply::structured("bid"),
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_clear(host: &mut Host, arguments: Value) -> ToolReply {
    match book_args(&arguments) {
        Ok((topic, bucket, epoch)) => match host.ads_clear(topic, bucket, epoch) {
            Ok(view) => ToolReply::structured(view),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_select(host: &mut Host, arguments: Value) -> ToolReply {
    match book_args(&arguments) {
        Ok((topic, bucket, epoch)) => match host.ads_select(topic, bucket, epoch) {
            Ok(view) => ToolReply::structured(view),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_settle(host: &mut Host, arguments: Value) -> ToolReply {
    match book_args(&arguments) {
        Ok((topic, bucket, epoch)) => match host.ads_settle(topic, bucket, epoch) {
            Ok(view) => ToolReply::structured(view),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

pub(crate) fn ads_hide(host: &mut Host, arguments: Value) -> ToolReply {
    match required_str(&arguments, "campaign") {
        Ok(campaign) => match host.ads_hide(campaign) {
            Ok(risk) => ToolReply::structured(json!({ "risk_milli": risk })),
            Err(error) => ToolReply::error(error.to_string()),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn book_args(arguments: &Value) -> Result<(&str, u8, u64), Error> {
    Ok((
        required_str(arguments, "topic")?,
        required_str(arguments, "bucket").and_then(parse_u8)?,
        required_str(arguments, "epoch").and_then(parse_u64)?,
    ))
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

fn parse_u8(text: &str) -> Result<u8, Error> {
    text.parse()
        .map_err(|_| Error::Codec("expected an unsigned integer"))
}
