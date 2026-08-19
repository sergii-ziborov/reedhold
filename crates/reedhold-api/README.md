# reedhold-api

The only crate a UI, `reedhold-host`, or `reedhold-mcp` should call.

Everything is synchronous. There is no Tokio. Strings and hex are the
boundary so Swift (and later UniFFI) can wrap this without seeing Rust
generics.

There is no Kotlin / JNI surface.

## Callers

- [reedhold-swift](https://github.com/sergii-ziborov/reedhold-swift) — iOS 14 / macOS 11
- [reedhold-host](https://github.com/sergii-ziborov/reedhold-host) — JSON HTTP process
- [reedhold-mcp](https://github.com/sergii-ziborov/reedhold) — MCP stdio in the protocol repo

## Surface

`Session`, `TalkNet`, `MeshSession`, `DurableSession`, `ChainSession`,
`RepSession`, `MarketSession`, `WorkSession`, `advertising_limits()`.
