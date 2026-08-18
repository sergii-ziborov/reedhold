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
