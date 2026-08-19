//! MCP adapter. No UI, no mesh.

#![forbid(unsafe_code)]

mod catalog;
mod host;
mod host_chain;
mod host_durable;
mod host_mesh;
mod host_talk;
mod schema;
mod server;
mod tools;
mod tools_chain;
mod tools_durable;
mod tools_mesh;
mod tools_store;
mod tools_talk;

pub use server::build_server;
