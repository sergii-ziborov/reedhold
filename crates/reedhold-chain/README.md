# reedhold-chain

Compact checkpoints. The chain stores roots, reputation epochs, and
market settlement — never private message bytes.

This crate is the in-process header log and light-client window.
A phone keeps 64 recent headers and Merkle proofs of selected heads.
It does not store DMs, photos, or search indexes.
