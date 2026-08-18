//! Domain-separated HKDF helpers.

use hkdf::Hkdf;
use reedhold_core::{Digest32, DomainTag, Error, Result};
use sha2::{Digest, Sha256};

pub(crate) fn expand(ikm: &[u8], tag: DomainTag) -> Result<[u8; 32]> {
    let hkdf = Hkdf::<Sha256>::new(None, ikm);
    let mut okm = [0_u8; 32];
    hkdf.expand(tag.as_bytes(), &mut okm)
        .map_err(|_| Error::Identity("hkdf expand failed"))?;
    Ok(okm)
}

pub(crate) fn digest(tag: DomainTag, material: &[u8]) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update(material);
    Digest32::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::expand;
    use reedhold_core::DomainTag;

    #[test]
    fn different_tags_diverge() {
        let seed = [7_u8; 32];
        let identity = expand(&seed, DomainTag::IdentityRoot).unwrap();
        let messaging = expand(&seed, DomainTag::MessagingRoot).unwrap();
        assert_ne!(identity, messaging);
    }
}
