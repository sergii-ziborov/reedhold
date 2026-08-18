# Reedhold

A social mesh that keeps holding.

Reedhold is a Rust protocol library for a social network that survives its
creator: recoverable cryptographic identity, signed events, bounded clients,
and state that does not live on a company server.

This repository is the **protocol workspace**, not a consumer app. iOS
(Swift), Android (Kotlin), Mac, Linux, and Windows all bind the same
sync crate: `reedhold-api`. AI agents use the same session through
`reedhold-mcp` (`mcport` + `blazingly-json`). The intended security
kernel is [Blindplane](https://github.com/sergii-ziborov/blindplane);
it stays a separate library.

> Prototype. Not independently audited. Do not use for real secrets yet.

## What is in scope

- recoverable identity (`MasterSeed` is random; a password only unlocks a vault)
- canonical binary encoding
- signed social events
- recovery manifests that can be stored on untrusted hosts
- ports for mesh, storage, chain, and a bounded client
- daily rotating transitional relays; the company site is optional
- a genesis advertising token that cannot control the mesh
- a sync host API that UI processes can wrap (no Tokio)
- an MCP server so an agent can create, restore, emit, and verify
- a local sealed store and k-of-n seed shares for reinstall / lost password
- an in-process mesh fabric: direct, rotating-relay store-and-forward, company optional
- Reed-Solomon durable objects: 4-of-6 for identity, survive a third of holders, then repair

## What is not in this repo yet

- a chat UI
- a production DHT / QUIC / BLE link (the routing fabric is in-process first)
- a token or speculative economy
- company servers as a source of truth

## Workspace

```text
crates/reedhold-core        identifiers, errors, domain tags
crates/reedhold-codec       deterministic binary encoding
crates/reedhold-identity    MasterSeed, roots, devices
crates/reedhold-recovery    vault + RecoveryManifest
crates/reedhold-event       SocialEvent kinds and envelopes
crates/reedhold-protocol    account lifecycle over the above
crates/reedhold-mesh        lottery, frames, in-process fabric (UDP/libp2p later)
crates/reedhold-ads         genesis advertising token (market only)
crates/reedhold-storage     erasure, placement, quotas, durable grid
crates/reedhold-chain       compact checkpoint types
crates/reedhold-client      light-client profile
crates/reedhold-store       local sealed manifest + signed event log
crates/reedhold-api         sync host session (Swift / Kotlin / desktop)
crates/reedhold-mcp         MCP stdio binary `reedhold`
crates/reedhold             public facade (does not include MCP)
```

Layering is one-way. Mesh, storage, and chain do not depend on each other.
UIs and agents never import mesh internals. They import `reedhold-api`.
Each UTC day the protocol redraws a small set of ordinary peers as
transitional sync hosts. Blocking the company site or yesterday's relays
does not halt the network.
The facade does not depend on MCP. No crate depends on the facade.

```sh
cargo run -p reedhold-mcp -- mcp
```

## Build

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Release gates:

- no `unsafe`
- no source file above 300 physical lines
- no function above 100 physical lines
- no dual `foo.rs` + `foo/` module layout
- Clippy pedantic, warnings denied

## Protocol prefix

```text
reedhold:identity:<hex>
```

Human handles (`@name`) are aliases. They are not the identity.

## License

MIT. See [LICENSE](LICENSE).
