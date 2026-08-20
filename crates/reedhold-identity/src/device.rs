//! Per-device keys. Each device writes its own append-only log.

use crate::derive::{digest, expand};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use reedhold_core::{DeviceId, DomainTag, Error, Result};

/// Authority that mints and verifies device keys for one identity.
#[derive(Clone, Debug)]
pub struct DeviceAuthority {
    device_root: [u8; 32],
}

impl DeviceAuthority {
    pub(crate) fn derive(master: &[u8; 32]) -> Result<Self> {
        Ok(Self {
            device_root: expand(master, DomainTag::DeviceRoot)?,
        })
    }

    /// Derive a deterministic device keypair from a local device secret.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when HKDF expansion fails.
    pub fn device_keys(&self, device_secret: &[u8; 32]) -> Result<DeviceKeys> {
        let mut ikm = [0_u8; 64];
        ikm[..32].copy_from_slice(&self.device_root);
        ikm[32..].copy_from_slice(device_secret);
        let secret = expand(&ikm, DomainTag::DeviceRoot)?;
        let signing = SigningKey::from_bytes(&secret);
        let public = signing.verifying_key().to_bytes();
        let id = DeviceId::from_digest(digest(DomainTag::DeviceRoot, &public));
        Ok(DeviceKeys { id, signing })
    }
}

/// Signing keys for one authorized device.
pub struct DeviceKeys {
    /// Device identifier derived from the verifying key.
    pub id: DeviceId,
    signing: SigningKey,
}

impl DeviceKeys {
    /// Ed25519 signature over `message`.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing.sign(message)
    }

    /// Verifying key bytes.
    #[must_use]
    pub fn public_bytes(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }

    /// Verify a signature produced by this device.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the signature is invalid.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        verify_device(&self.public_bytes(), message, signature)
    }
}

/// Verify a device signature from the public key bytes.
///
/// # Errors
///
/// Returns [`Error::Identity`] when the key or signature is invalid.
pub fn verify_device(public: &[u8; 32], message: &[u8], signature: &Signature) -> Result<()> {
    let verifying = VerifyingKey::from_bytes(public)
        .map_err(|_| Error::Identity("invalid device public key"))?;
    verifying
        .verify(message, signature)
        .map_err(|_| Error::Identity("device signature rejected"))
}

#[cfg(test)]
mod tests {
    use super::DeviceAuthority;

    #[test]
    fn a_device_id_reveals_nothing_about_the_identity() {
        // Both branches grow from the same MasterSeed, so the question is
        // whether one can be walked back to the other. It cannot: each hop is
        // an HKDF expansion or a SHA-256 digest, and neither inverts.
        let master = [3_u8; 32];
        let authority = DeviceAuthority::derive(&master).unwrap();
        let phone = authority.device_keys(&[1_u8; 32]).unwrap();
        let laptop = authority.device_keys(&[2_u8; 32]).unwrap();

        let identity_root =
            crate::derive::expand(&master, reedhold_core::DomainTag::IdentityRoot).unwrap();
        let device_root =
            crate::derive::expand(&master, reedhold_core::DomainTag::DeviceRoot).unwrap();
        assert_ne!(identity_root, device_root, "roots must not collide");

        // Two devices of one person share no bytes an observer could join on.
        assert_ne!(phone.id, laptop.id);
        assert_ne!(phone.public_bytes(), laptop.public_bytes());
        assert_ne!(phone.id.as_digest().as_bytes(), &identity_root);
        assert_ne!(phone.id.as_digest().as_bytes(), &device_root);

        // The id is exactly the digest of the public key and nothing more.
        let expected = super::digest(reedhold_core::DomainTag::DeviceRoot, &phone.public_bytes());
        assert_eq!(phone.id.as_digest(), &expected);
    }

    #[test]
    fn device_can_sign_and_verify() {
        let authority = DeviceAuthority::derive(&[9_u8; 32]).unwrap();
        let keys = authority.device_keys(&[4_u8; 32]).unwrap();
        let signature = keys.sign(b"hello");
        keys.verify(b"hello", &signature).unwrap();
        assert!(keys.verify(b"other", &signature).is_err());
    }
}
