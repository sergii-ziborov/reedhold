//! Seal a `MasterSeed` behind Argon2id and XChaCha20-Poly1305.

use crate::params::KdfParams;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use reedhold_core::{Error, Result};
use reedhold_identity::MasterSeed;

const NONCE_LEN: usize = 24;
const SALT_LEN: usize = 16;

/// Encrypted master seed plus the salt and nonce needed to open it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedSeed {
    /// Argon2id salt.
    pub salt: [u8; SALT_LEN],
    /// XChaCha20-Poly1305 nonce.
    pub nonce: [u8; NONCE_LEN],
    /// Ciphertext including the Poly1305 tag.
    pub ciphertext: Vec<u8>,
    /// Parameters used to derive the unlock key.
    pub params: KdfParams,
}

/// Seal `seed` under `password`.
///
/// # Errors
///
/// Returns [`Error::Recovery`] or [`Error::Entropy`] on KDF or AEAD failure.
pub fn seal_seed(password: &[u8], seed: &MasterSeed, params: KdfParams) -> Result<SealedSeed> {
    let mut salt = [0_u8; SALT_LEN];
    getrandom_bytes(&mut salt)?;
    let mut nonce = [0_u8; NONCE_LEN];
    getrandom_bytes(&mut nonce)?;
    let key = derive_key(password, &salt, params)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), seed.as_bytes().as_slice())
        .map_err(|_| Error::Recovery("seal failed"))?;
    Ok(SealedSeed {
        salt,
        nonce,
        ciphertext,
        params,
    })
}

/// Open a sealed seed with `password`.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when the password or ciphertext is wrong.
pub fn unseal_seed(password: &[u8], sealed: &SealedSeed) -> Result<MasterSeed> {
    let key = derive_key(password, &sealed.salt, sealed.params)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let plain = cipher
        .decrypt(
            XNonce::from_slice(&sealed.nonce),
            sealed.ciphertext.as_ref(),
        )
        .map_err(|_| Error::Recovery("unseal failed"))?;
    let bytes: [u8; 32] = plain
        .try_into()
        .map_err(|_| Error::Recovery("unsealed seed has the wrong length"))?;
    Ok(MasterSeed::from_bytes(bytes))
}

fn derive_key(password: &[u8], salt: &[u8], params: KdfParams) -> Result<[u8; 32]> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|_| Error::Recovery("invalid argon2 params"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = [0_u8; 32];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| Error::Recovery("argon2 failed"))?;
    Ok(key)
}

fn getrandom_bytes(buffer: &mut [u8]) -> Result<()> {
    getrandom::getrandom(buffer).map_err(|_| Error::Entropy)
}

#[cfg(test)]
mod tests {
    use super::{seal_seed, unseal_seed};
    use crate::params::KdfParams;
    use reedhold_identity::MasterSeed;

    #[test]
    fn wrong_password_fails() {
        let seed = MasterSeed::from_bytes([5_u8; 32]);
        let sealed = seal_seed(b"correct", &seed, KdfParams::TEST).unwrap();
        assert!(unseal_seed(b"wrong", &sealed).is_err());
        let opened = unseal_seed(b"correct", &sealed).unwrap();
        assert_eq!(opened.as_bytes(), seed.as_bytes());
    }
}
