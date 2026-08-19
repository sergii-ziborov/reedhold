//! Domain-separated SHA-256 for chain objects.

use reedhold_core::{Digest32, DomainTag};
use sha2::{Digest, Sha256};

pub(crate) fn digest(tag: DomainTag, parts: &[&[u8]]) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    for part in parts {
        hasher.update(part);
    }
    Digest32::from_bytes(hasher.finalize().into())
}
