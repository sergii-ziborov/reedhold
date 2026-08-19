//! `mcport` server composition.

use crate::catalog::{with_account_tools, with_chain, with_store_and_mesh, with_talk};
use crate::catalog_ads::with_ads;
use crate::catalog_rep::with_rep;
use crate::catalog_work::with_work;
use crate::host::Host;
use mcport::McpServer;

/// Build the Reedhold MCP server.
#[must_use]
pub fn build_server() -> McpServer<Host> {
    let server = McpServer::with_state("reedhold", env!("CARGO_PKG_VERSION"), Host::default())
        .instructions(
            "Reedhold protocol session. Create or restore an account, then emit or verify signed events. No company server is required.",
        )
        .strict_schemas();
    with_work(with_ads(with_rep(with_chain(with_talk(
        with_store_and_mesh(with_account_tools(server)),
    )))))
}
