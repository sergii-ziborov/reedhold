//! MCP adapter. No UI, no mesh.

#![forbid(unsafe_code)]

mod host;
mod server;
mod tools;

pub use server::build_server;
