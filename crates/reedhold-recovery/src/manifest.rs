//! Content-addressed recovery manifest.

use crate::params::KdfParams;
use crate::vault::SealedSeed;
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Error, IdentityId, NetworkId, Result};

const MANIFEST_TAG: u8 = 0x01;

/// Replicated recovery object. Does not contain a plaintext seed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryManifest {
    /// Network this identity belongs to.
    pub network: NetworkId,
    /// Identity that can be restored from this manifest.
    pub identity: IdentityId,
    /// Vault epoch. Incremented on password change.
    pub epoch: u64,
    /// Sealed master seed.
    pub sealed: SealedSeed,
}

impl RecoveryManifest {
    /// Encode the manifest canonically.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a field cannot be encoded.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(MANIFEST_TAG);
        writer.write_u16(reedhold_core::PROTOCOL_VERSION);
        writer.write_bytes(self.network.as_str().as_bytes())?;
        writer.write_digest32(self.identity.as_digest().as_bytes());
        writer.write_u64(self.epoch);
        writer.write_bytes(&self.sealed.salt)?;
        writer.write_bytes(&self.sealed.nonce)?;
        writer.write_bytes(&self.sealed.ciphertext)?;
        writer.write_bytes(&self.sealed.params.to_bytes())?;
        Ok(writer.finish())
    }

    /// Decode a canonical manifest.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] or [`Error::Recovery`] on a malformed buffer.
    pub fn decode(bytes: &[u8], expected_network: NetworkId) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != MANIFEST_TAG {
            return Err(Error::Recovery("unknown manifest tag"));
        }
        if reader.read_u16()? != reedhold_core::PROTOCOL_VERSION {
            return Err(Error::Recovery("unsupported protocol version"));
        }
        let network_raw = reader.read_bytes()?;
        if network_raw != expected_network.as_str().as_bytes() {
            return Err(Error::Recovery("network mismatch"));
        }
        let identity =
            IdentityId::from_digest(reedhold_core::Digest32::from_bytes(reader.read_digest32()?));
        let epoch = reader.read_u64()?;
        let sealed = read_sealed(&mut reader)?;
        reader.finish()?;
        Ok(Self {
            network: expected_network,
            identity,
            epoch,
            sealed,
        })
    }
}

fn read_sealed(reader: &mut Reader<'_>) -> Result<SealedSeed> {
    let salt = take_len::<16>(reader.read_bytes()?)?;
    let nonce = take_len::<24>(reader.read_bytes()?)?;
    let ciphertext = reader.read_bytes()?.to_vec();
    let params = take_len::<12>(reader.read_bytes()?)?;
    Ok(SealedSeed {
        salt,
        nonce,
        ciphertext,
        params: KdfParams::from_bytes(params),
    })
}

fn take_len<const N: usize>(bytes: &[u8]) -> Result<[u8; N]> {
    bytes
        .try_into()
        .map_err(|_| Error::Recovery("manifest field has the wrong length"))
}
