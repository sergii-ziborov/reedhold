//! JSON schemas for MCP tools.

use mcport::{Map, Value, json};

pub(crate) fn object_schema(required: &[&str]) -> Value {
    let mut properties = Map::new();
    for field in required {
        properties.insert((*field).into(), json!({ "type": "string" }));
    }
    let required = required
        .iter()
        .map(|field| Value::String((*field).to_owned()))
        .collect();
    json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": Value::Array(required),
        "additionalProperties": false
    })
}

pub(crate) fn sync_plan_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "epoch": { "type": "string" },
            "prior_commit": { "type": "string" },
            "candidates": { "type": "array", "items": { "type": "string" } },
            "company": { "type": "string" }
        },
        "required": ["epoch", "candidates"],
        "additionalProperties": false
    })
}

pub(crate) fn holders_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "holders": { "type": "array", "items": { "type": "string" } },
            "company": { "type": "string" }
        },
        "required": ["holders"],
        "additionalProperties": false
    })
}

pub(crate) fn combine_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "threshold": { "type": "string" },
            "password": { "type": "string" },
            "device_secret": { "type": "string" },
            "shares": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "index": { "type": "string" },
                        "body_hex": { "type": "string" }
                    },
                    "required": ["index", "body_hex"]
                }
            }
        },
        "required": ["threshold", "password", "device_secret", "shares"],
        "additionalProperties": false
    })
}

pub(crate) fn chain_commit_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "epoch": { "type": "string" },
            "identity": { "type": "string" },
            "groups": { "type": "string" },
            "storage": { "type": "string" }
        },
        "required": ["epoch"],
        "additionalProperties": false
    })
}

pub(crate) fn chain_prove_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "leaves": { "type": "array", "items": { "type": "string" } },
            "index": { "type": "string" }
        },
        "required": ["leaves", "index"],
        "additionalProperties": false
    })
}

pub(crate) fn chain_verify_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "leaf": { "type": "string" },
            "root": { "type": "string" },
            "index": { "type": "string" },
            "siblings": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["leaf", "root", "index", "siblings"],
        "additionalProperties": false
    })
}

pub(crate) fn rep_seed_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "identity": { "type": "string" },
            "continuity": { "type": "string" },
            "social": { "type": "string" },
            "content": { "type": "string" },
            "curation": { "type": "string" }
        },
        "required": ["identity", "continuity", "social", "content", "curation"],
        "additionalProperties": false
    })
}

pub(crate) fn rep_react_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "author": { "type": "string" },
            "target": { "type": "string" },
            "kind": { "type": "string" },
            "now": { "type": "string" },
            "cluster": { "type": "string" }
        },
        "required": ["author", "target", "kind", "now"],
        "additionalProperties": false
    })
}
