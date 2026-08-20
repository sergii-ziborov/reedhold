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

/// Seal `seed` under `password` alone. Prefer [`seal_seed_with`].
///
/// # Errors
///
/// Returns [`Error::Recovery`] or [`Error::Entropy`] on KDF or AEAD failure.
pub fn seal_seed(password: &[u8], seed: &MasterSeed, params: KdfParams) -> Result<SealedSeed> {
    seal_seed_with(password, None, seed, params)
}

/// Seal `seed` under `password` plus an optional second factor.
///
/// The factor is Argon2's secret input — a pepper. A recovery blob is fetched
/// from an untrusted mesh, so an attacker holding it can guess passwords
/// offline for as long as they like. Argon2id prices each guess; only a second
/// secret they do not have takes the guessing off the table entirely.
///
/// # Errors
///
/// Returns [`Error::Recovery`] or [`Error::Entropy`] on KDF or AEAD failure.
pub fn seal_seed_with(
    password: &[u8],
    factor: Option<&[u8]>,
    seed: &MasterSeed,
    params: KdfParams,
) -> Result<SealedSeed> {
    let mut salt = [0_u8; SALT_LEN];
    getrandom_bytes(&mut salt)?;
    let mut nonce = [0_u8; NONCE_LEN];
    getrandom_bytes(&mut nonce)?;
    let key = derive_key(password, factor, &salt, params)?;
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

/// Open a sealed seed with `password` alone.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when the password or ciphertext is wrong.
pub fn unseal_seed(password: &[u8], sealed: &SealedSeed) -> Result<MasterSeed> {
    unseal_seed_with(password, None, sealed)
}

/// Open a sealed seed with `password` and the factor it was sealed under.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when either input is wrong.
pub fn unseal_seed_with(
    password: &[u8],
    factor: Option<&[u8]>,
    sealed: &SealedSeed,
) -> Result<MasterSeed> {
    let key = derive_key(password, factor, &sealed.salt, sealed.params)?;
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

fn derive_key(
    password: &[u8],
    factor: Option<&[u8]>,
    salt: &[u8],
    params: KdfParams,
) -> Result<[u8; 32]> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|_| Error::Recovery("invalid argon2 params"))?;
    let argon = match factor {
        Some(secret) if !secret.is_empty() => {
            Argon2::new_with_secret(secret, Algorithm::Argon2id, Version::V0x13, argon_params)
                .map_err(|_| Error::Recovery("invalid recovery factor"))?
        }
        _ => Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params),
    };
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
    use super::{seal_seed, seal_seed_with, unseal_seed, unseal_seed_with};
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

    #[test]
    fn a_second_factor_takes_offline_guessing_off_the_table() {
        let seed = MasterSeed::from_bytes([6_u8; 32]);
        let factor = [0xa5_u8; 32];
        // A short password on purpose: the point is that it stops mattering.
        let sealed = seal_seed_with(b"1234", Some(&factor), &seed, KdfParams::TEST).unwrap();

        assert!(
            unseal_seed(b"1234", &sealed).is_err(),
            "the right password alone must not open a two-factor vault"
        );
        assert!(unseal_seed_with(b"1234", Some(&[0_u8; 32]), &sealed).is_err());
        assert_eq!(
            unseal_seed_with(b"1234", Some(&factor), &sealed)
                .unwrap()
                .as_bytes(),
            seed.as_bytes()
        );
    }

    #[test]
    fn the_production_profile_is_not_the_test_profile() {
        let cheap = KdfParams::TEST;
        let real = KdfParams::INTERACTIVE;
        assert!(real.memory_kib >= 64 * 1024, "RFC 9106 asks for 64 MiB");
        assert!(real.iterations >= 3);
        assert!(real.memory_kib > cheap.memory_kib);
        assert!(real.iterations > cheap.iterations);
    }
}
