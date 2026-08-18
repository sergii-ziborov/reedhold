# Security

Reedhold is a prototype. It is **not independently audited**.

Do not store real private messages, recovery secrets, or production keys
in a build from this repository until an external review says otherwise.

## What this tree may do today

- derive identity material from a random `MasterSeed`
- seal that seed behind Argon2id + XChaCha20-Poly1305 for local tests
- sign social events with Ed25519 device keys

These constructions use standard crates. They are a stand-in until
[Blindplane](https://github.com/sergii-ziborov/blindplane) is wired as the
security kernel. The Blindplane boundary must stay outside this workspace.

## Report a vulnerability

Open a private GitHub security advisory on
[sergii-ziborov/reedhold](https://github.com/sergii-ziborov/reedhold),
or email the address in the crate metadata.

Please do not file a public issue for a key-recovery or decryption bug.
