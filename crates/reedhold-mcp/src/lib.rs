//! MCP adapter. No UI, no mesh.

#![forbid(unsafe_code)]

mod catalog;
mod host;
mod host_durable;
mod host_mesh;
mod schema;
mod server;
mod tools;
mod tools_durable;
mod tools_mesh;
mod tools_store;

pub use server::build_server;
