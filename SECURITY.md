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

## Addressing

A device id is `SHA-256(tag || device_public)` and reveals nothing about the
account. Device keys and the identity root are separate HKDF branches of the
`MasterSeed`, and neither hash inverts, so there is no path from a device id
back to an `IdentityId`. The test
`a_device_id_reveals_nothing_about_the_identity` holds that line.

Conversations no longer travel addressed to an identity. A DM or group message
is posted to `H(tag || shared_secret || epoch)` — an address only the endpoints
can derive, rotating every six hours. Author, device key, messaging key and the
signed event all move inside the ciphertext, so a carrier sees an address it
cannot attribute and bytes it cannot read. Two epochs of one conversation share
no prefix an observer could group on.

Group invites still go identity-addressed: they are what delivers the group key
that the group's mailbox is derived from.

### What this does not cover yet

A stranger cannot open a conversation. A pairwise address needs both halves of
a shared secret, and a first message arrives before the recipient has any way
to derive it. Closing that needs a sealed box under the recipient's published
key — `delivery_topic` is the address side of it; the crypto side is not built.
Until then a DM requires that both sides already hold each other's keys.

## Passwords

A recovery blob is fetched from an untrusted mesh, so anyone holding it can
guess passwords offline for as long as they like. Two things price that.

The vault is sealed under Argon2id at `KdfParams::INTERACTIVE` — RFC 9106's
second recommended option, 64 MiB with three passes. It was previously sealed
at the test profile, 8 MiB and a single pass, which costs an attacker almost
nothing per guess.

`seal_seed_with` accepts a second factor as Argon2's secret input. With one
set, the password alone does not open the vault no matter how many guesses an
attacker makes, because they are missing key material rather than solving a
puzzle. A username is not such a factor: it is public, and the salt already
individualises the KDF.

## Report a vulnerability

Open a private GitHub security advisory on
[sergii-ziborov/reedhold](https://github.com/sergii-ziborov/reedhold),
or email the address in the crate metadata.

Please do not file a public issue for a key-recovery or decryption bug.
