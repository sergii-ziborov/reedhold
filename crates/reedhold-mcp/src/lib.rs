//! MCP adapter. No UI, no mesh.

#![forbid(unsafe_code)]

mod host;
mod server;
mod tools;
mod tools_store;

pub use server::build_server;
