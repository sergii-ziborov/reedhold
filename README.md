# Reedhold

A social mesh that keeps holding.

Reedhold is a Rust **protocol library** for a social network that survives its
creator: recoverable cryptographic identity, signed events, bounded clients,
and state that does not live on a company server.

This repository is the protocol workspace, not a consumer app.

| Repo | Role |
| --- | --- |
| [reedhold](https://github.com/sergii-ziborov/reedhold) | Protocol crates (`reedhold-api` is the only host surface) |
| [reedhold-host](https://github.com/sergii-ziborov/reedhold-host) | Sync JSON HTTP process wrapping `reedhold-api` (no Tokio in the kernel) |
| [reedhold-swift](https://github.com/sergii-ziborov/reedhold-swift) | iOS 14+ / macOS 11+ client |
| [reedhold-app](https://github.com/sergii-ziborov/reedhold-app) | Native desktop client (`eframe`/`egui`), calls `reedhold-api` in-process |
| [reedhold-site](https://github.com/sergii-ziborov/reedhold-site) | Public site + web app ([reedhold.com](https://reedhold.com)) |
| [reedhold-mcp](https://github.com/sergii-ziborov/reedhold) (`reedhold-mcp` crate) | Agent MCP stdio in this workspace |

There is no Kotlin / Android repo. UIs call `reedhold-api` (later UniFFI) or
`reedhold-host`. They never import mesh, storage, or MCP internals.

The intended security kernel is
[Blindplane](https://github.com/sergii-ziborov/blindplane); it stays a
separate library.

> Prototype. Not independently audited. Do not use for real secrets yet.

## What the protocol already has

- recoverable identity (`MasterSeed` is random; a password only unlocks a vault)
- canonical binary encoding
- signed social events (`post` through `group_leave`)
- recovery manifests for untrusted hosts; k-of-n Shamir shares
- local sealed store and reinstall proof
- daily rotating transitional relays; company host optional; blocking it is not fatal
- genesis advertising token: market rights only, not network control
- in-process mesh fabric: direct, rotating-relay store-and-forward
- bounded peer table (256 peers) scoring peers by measured uptime and delivery
  success, not by age: a node that has been around for years but is usually
  dark loses to a newcomer that answers
- greedy multi-hop routing over XOR distance, capped at 12 hops; each step must
  land strictly closer to the target, which is what makes the walk terminate
- `Route::Remote` hands a packet back to the host when the next hop lives in
  another process
- DMs and small groups: pairwise X25519, shared epoch keys, leave rotates the key
- the author keeps their own copy: the fabric only carries mail to other people,
  so a sender who kept nothing could never reread what they wrote
- a late peer joins a running fabric instead of rebuilding it, because a rebuild
  drops every relay queue and loses mail nobody has collected yet
- public nicknames are aliases only: they are not identity and never enter signed talk bytes
- a released nick is retired for a year, so a stranger cannot inherit the name
  people knew you by; a contact records the nick you had, and a later rename
  shows as "@bob, was @alice"
- privacy is a session policy, not a transport rule: who may write to you,
  blocked identities, archived conversations, and a requests tray for strangers
- public topic rooms: slug is a local label; posts carry identity hex
- owner-admin groups: invite and remove rotate the epoch key
- Reed-Solomon durable objects: 4-of-6, survive a third of holders, then repair
- compact chain headers: identity/group/storage Merkle roots, 64-header light window, no message bytes
- reputation v0: likes mature, cluster pumps are cheap, influence budget, not transferable
- attention-market sandbox: batch uniform-price on `(topic, bucket, epoch)`, no user-id targeting
- proof of contribution: storage/relay/repair mint credits; history stays; popularity is not consensus
- sync host API (`reedhold-api`) and MCP (`reedhold-mcp` via `mcport` + `blazingly-json`)

## What is not in this repo

- a production DHT / QUIC / BLE link (the fabric is in-process first)
- MLS for large groups
- a token or speculative economy (sandbox credits only)
- company servers as a source of truth
- consumer UIs (see sibling repos)

## Host API (`reedhold-api`)

Swift, desktop, the HTTP host, and MCP all share this crate. Strings and hex
only. No Tokio.

| Type | Job |
| --- | --- |
| `Session` | create / restore / emit / verify / sealed DM / password change / Shamir split |
| HTTP (`reedhold-host`) | session + talk + durable + chain + rep + ads + work on `127.0.0.1:4783` |
| `TalkNet` | DMs and small groups over the fabric |
| `MeshSession` | in-process routing (direct / relay / held) |
| `DurableSession` | erasure grid put/get/kill/repair |
| `ChainSession` | compact headers, Merkle proofs |
| `RepSession` | mature reactions, influence budget |
| `MarketSession` | topic/bucket auctions |
| `WorkSession` | contribution credits |
| `advertising_limits()` | genesis token capability mask |

Identity URI:

```text
reedhold:identity:<hex>
```

Human handles (`@name`) are aliases. They are not the identity.

## Workspace

```text
crates/reedhold-core        identifiers, errors, domain tags
crates/reedhold-codec       deterministic binary encoding
crates/reedhold-identity    MasterSeed, roots, devices
crates/reedhold-recovery    vault + RecoveryManifest
crates/reedhold-event       SocialEvent kinds and envelopes
crates/reedhold-protocol    account lifecycle over the above
crates/reedhold-rep         mature reactions and influence budget
crates/reedhold-mesh        lottery, frames, in-process fabric
crates/reedhold-ads         genesis token + attention-market sandbox
crates/reedhold-storage     erasure, placement, quotas, durable grid
crates/reedhold-chain       compact checkpoint types
crates/reedhold-client      light-client profile
crates/reedhold-store       local sealed manifest + signed event log
crates/reedhold-work        proof of contribution and sandbox credits
crates/reedhold-api         sync host session (Swift / desktop / HTTP host / MCP)
crates/reedhold-mcp         MCP stdio binary `reedhold`
crates/reedhold             public facade (does not include MCP)
```

Layering is one-way. Mesh, storage, and chain do not depend on each other.
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

## License

MIT. See [LICENSE](LICENSE).
