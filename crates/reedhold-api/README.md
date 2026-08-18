# reedhold-api

The only crate iOS, Android, desktop, and the MCP server should call.

Everything is synchronous. There is no Tokio. Strings and hex are the
boundary so UniFFI / JNI / Swift can wrap this later without seeing
Rust generics.
