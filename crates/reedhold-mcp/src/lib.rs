//! MCP adapter. No UI, no mesh.

#![forbid(unsafe_code)]

mod catalog;
mod catalog_ads;
mod catalog_rep;
mod catalog_work;
mod host;
mod host_ads;
mod host_chain;
mod host_durable;
mod host_mesh;
mod host_rep;
mod host_talk;
mod host_work;
mod schema;
mod server;
mod tools;
mod tools_ads;
mod tools_chain;
mod tools_durable;
mod tools_mesh;
mod tools_rep;
mod tools_store;
mod tools_talk;
mod tools_work;

pub use server::build_server;
