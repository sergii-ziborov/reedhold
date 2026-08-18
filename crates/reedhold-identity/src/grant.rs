//! Device authorization grant signed by the identity root.

use crate::device::DeviceKeys;
use crate::root::IdentityRoot;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use reedhold_codec::{Reader, Writer};
use reedhold_core::{DeviceId, Error, IdentityId, Result};

const GRANT_TAG: u8 = 0x21;

/// A device is allowed to emit events for this identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceGrant {
    /// Account that issued the grant.
    pub identity: IdentityId,
    /// Authorized device.
    pub device: DeviceId,
    /// Device verifying key.
    pub device_public: [u8; 32],
    /// Grant epoch. Incremented on revoke/reissue.
    pub epoch: u64,
    /// Identity-root signature over the canonical body.
    pub signature: [u8; 64],
}

impl DeviceGrant {
    /// Issue a grant for `device`.
    #[must_use]
    pub fn issue(root: &IdentityRoot, device: &DeviceKeys, epoch: u64) -> Self {
        let mut grant = Self {
            identity: root.identity,
            device: device.id,
            device_public: device.public_bytes(),
            epoch,
            signature: [0_u8; 64],
        };
        let body = encode_body(&grant);
        grant.signature = root.sign(&body).to_bytes();
        grant
    }

    /// Canonical encoding including the signature.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when encoding fails.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_bytes(&encode_body(self))?;
        writer.write_bytes(&self.signature)?;
        Ok(writer.finish())
    }

    /// Decode and verify against the identity-root public key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Identity`] when the grant is malformed or the
    /// signature does not match `root_public`.
    pub fn decode_verify(bytes: &[u8], root_public: &[u8; 32]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let body = reader.read_bytes()?.to_vec();
        let signature = take64(reader.read_bytes()?)?;
        reader.finish()?;
        let grant = decode_body(&body, signature)?;
        let verifying = VerifyingKey::from_bytes(root_public)
            .map_err(|_| Error::Identity("invalid root public key"))?;
        verifying
            .verify(&body, &Signature::from_bytes(&grant.signature))
            .map_err(|_| Error::Identity("device grant signature rejected"))?;
        Ok(grant)
    }
}

fn encode_body(grant: &DeviceGrant) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.write_u8(GRANT_TAG);
    writer.write_digest32(grant.identity.as_digest().as_bytes());
    writer.write_digest32(grant.device.as_digest().as_bytes());
    writer.write_digest32(&grant.device_public);
    writer.write_u64(grant.epoch);
    writer.finish()
}

fn decode_body(bytes: &[u8], signature: [u8; 64]) -> Result<DeviceGrant> {
    let mut reader = Reader::new(bytes);
    if reader.read_u8()? != GRANT_TAG {
        return Err(Error::Identity("unknown grant tag"));
    }
    let identity =
        IdentityId::from_digest(reedhold_core::Digest32::from_bytes(reader.read_digest32()?));
    let device =
        DeviceId::from_digest(reedhold_core::Digest32::from_bytes(reader.read_digest32()?));
    let device_public = reader.read_digest32()?;
    let epoch = reader.read_u64()?;
    reader.finish()?;
    Ok(DeviceGrant {
        identity,
        device,
        device_public,
        epoch,
        signature,
    })
}

fn take64(bytes: &[u8]) -> Result<[u8; 64]> {
    bytes
        .try_into()
        .map_err(|_| Error::Identity("grant signature has the wrong length"))
}

#[cfg(test)]
mod tests {
    use super::DeviceGrant;
    use crate::MasterSeed;

    #[test]
    fn grant_round_trips() {
        let seed = MasterSeed::from_bytes([8_u8; 32]);
        let bundle = seed.unlock().unwrap();
        let device = bundle.devices.device_keys(&[1_u8; 32]).unwrap();
        let grant = DeviceGrant::issue(&bundle.root, &device, 1);
        let encoded = grant.encode().unwrap();
        let verified = DeviceGrant::decode_verify(&encoded, &bundle.root.public).unwrap();
        assert_eq!(verified.device, device.id);
        assert!(DeviceGrant::decode_verify(&encoded, &[0_u8; 32]).is_err());
    }
}
