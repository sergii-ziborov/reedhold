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
- a sync host API that UI processes can wrap (no Tokio)
- an MCP server so an agent can create, restore, emit, and verify

## What is not in this repo yet

- a chat UI
- a live DHT / gossip implementation
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
crates/reedhold-mesh        discovery / transport ports
crates/reedhold-storage     durability classes and node budgets
crates/reedhold-chain       compact checkpoint types
crates/reedhold-client      light-client profile
crates/reedhold-api         sync host session (Swift / Kotlin / desktop)
crates/reedhold-mcp         MCP stdio binary `reedhold`
crates/reedhold             public facade (does not include MCP)
```

Layering is one-way. Mesh, storage, and chain do not depend on each other.
UIs and agents never import mesh. They import `reedhold-api`.
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
