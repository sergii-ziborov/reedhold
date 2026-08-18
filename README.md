# Reedhold

A social mesh that keeps holding.

Reedhold is a Rust protocol library for a social network that survives its
creator: recoverable cryptographic identity, signed events, bounded clients,
and state that does not live on a company server.

This repository is the **protocol workspace**, not a consumer app. Clients for
desktop, iOS, and Android will bind to these crates later. The intended
security kernel is [Blindplane](https://github.com/sergii-ziborov/blindplane);
it stays a separate library.

> Prototype. Not independently audited. Do not use for real secrets yet.

## What is in scope

- recoverable identity (`MasterSeed` is random; a password only unlocks a vault)
- canonical binary encoding
- signed social events
- recovery manifests that can be stored on untrusted hosts
- ports for mesh, storage, chain, and a bounded client

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
crates/reedhold             public facade
```

Layering is one-way. Mesh, storage, and chain do not depend on each other.
The facade is the only crate that sees every layer. No crate depends on the
facade.

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
