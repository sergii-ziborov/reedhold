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

## Known: the transport still addresses by identity

A device id is `SHA-256(tag || device_public)` and reveals nothing about the
account. Device keys and the identity root are separate HKDF branches of the
`MasterSeed`, and neither hash inverts, so there is no path from a device id
back to an `IdentityId`. The test
`a_device_id_reveals_nothing_about_the_identity` holds that line.

The identity itself is another matter. `TalkPacket` carries `author` in the
clear, and the fabric uses identity hex as the peer address. Every relay on a
path therefore sees "identity A is talking to identity B". Message bodies are
sealed; the social graph is not.

The fix is specified and not yet built: rotating mailbox topics derived from
the pairwise shared secret, `H(shared_secret || epoch || "mailbox")`, so an
outsider sees unrelated random ids rather than a conversation.

## Report a vulnerability

Open a private GitHub security advisory on
[sergii-ziborov/reedhold](https://github.com/sergii-ziborov/reedhold),
or email the address in the crate metadata.

Please do not file a public issue for a key-recovery or decryption bug.
