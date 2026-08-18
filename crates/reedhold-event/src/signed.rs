//! Signed event envelope.

use crate::kind::EventKind;
use ed25519_dalek::Signature;
use reedhold_codec::{Reader, Writer};
use reedhold_core::{ContentId, DeviceId, Digest32, Error, IdentityId, NetworkId, Result};
use reedhold_identity::{DeviceKeys, verify_device};
use sha2::{Digest, Sha256};

const EVENT_TAG: u8 = 0x10;

/// Content-address a payload with the protocol domain tag.
#[must_use]
pub fn content_id(payload: &[u8]) -> ContentId {
    let mut hasher = Sha256::new();
    hasher.update(reedhold_core::DomainTag::Content.as_bytes());
    hasher.update(payload);
    ContentId::from_digest(Digest32::from_bytes(hasher.finalize().into()))
}

/// Signed social event. Payload bytes live behind `payload`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedEvent {
    /// Network isolation.
    pub network: NetworkId,
    /// Author identity.
    pub author: IdentityId,
    /// Device that produced the event.
    pub device: DeviceId,
    /// Monotonic per-device sequence.
    pub sequence: u64,
    /// Event kind.
    pub kind: EventKind,
    /// Content-addressed payload.
    pub payload: ContentId,
    /// Ed25519 signature over the canonical unsigned body.
    pub signature: [u8; 64],
}

impl SignedEvent {
    /// Canonical unsigned body used for signing.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when encoding fails.
    pub fn unsigned_body(&self) -> Result<Vec<u8>> {
        encode_body(self)
    }

    /// Encode the signed event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when encoding fails.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_bytes(&self.unsigned_body()?)?;
        writer.write_bytes(&self.signature)?;
        Ok(writer.finish())
    }

    /// Decode and verify a signed event.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the structure or signature is invalid.
    pub fn decode_verify(
        bytes: &[u8],
        network: NetworkId,
        device_public: &[u8; 32],
    ) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let body = reader.read_bytes()?.to_vec();
        let signature_raw = reader.read_bytes()?;
        reader.finish()?;
        let signature = take_signature(signature_raw)?;
        let event = decode_body(&body, network, signature)?;
        let dalek = Signature::from_bytes(&event.signature);
        verify_device(device_public, &body, &dalek).map_err(|_| Error::Event("bad signature"))?;
        Ok(event)
    }
}

/// Sign a new event with a device key.
///
/// # Errors
///
/// Returns [`Error::Codec`] when the body cannot be encoded.
pub fn sign_event(
    network: NetworkId,
    author: IdentityId,
    device: &DeviceKeys,
    sequence: u64,
    kind: EventKind,
    payload: ContentId,
) -> Result<SignedEvent> {
    let mut event = SignedEvent {
        network,
        author,
        device: device.id,
        sequence,
        kind,
        payload,
        signature: [0_u8; 64],
    };
    let body = encode_body(&event)?;
    event.signature = device.sign(&body).to_bytes();
    Ok(event)
}

fn encode_body(event: &SignedEvent) -> Result<Vec<u8>> {
    let mut writer = Writer::new();
    writer.write_u8(EVENT_TAG);
    writer.write_u16(reedhold_core::PROTOCOL_VERSION);
    writer.write_bytes(event.network.as_str().as_bytes())?;
    writer.write_digest32(event.author.as_digest().as_bytes());
    writer.write_digest32(event.device.as_digest().as_bytes());
    writer.write_u64(event.sequence);
    writer.write_u16(event.kind.as_u16());
    writer.write_digest32(event.payload.as_digest().as_bytes());
    Ok(writer.finish())
}

fn decode_body(bytes: &[u8], network: NetworkId, signature: [u8; 64]) -> Result<SignedEvent> {
    let mut reader = Reader::new(bytes);
    if reader.read_u8()? != EVENT_TAG {
        return Err(Error::Event("unknown event tag"));
    }
    if reader.read_u16()? != reedhold_core::PROTOCOL_VERSION {
        return Err(Error::Event("unsupported protocol version"));
    }
    if reader.read_bytes()? != network.as_str().as_bytes() {
        return Err(Error::Event("network mismatch"));
    }
    let author = IdentityId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
    let device = DeviceId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
    let sequence = reader.read_u64()?;
    let kind = EventKind::from_u16(reader.read_u16()?).ok_or(Error::Event("unknown kind"))?;
    let payload = ContentId::from_digest(Digest32::from_bytes(reader.read_digest32()?));
    reader.finish()?;
    Ok(SignedEvent {
        network,
        author,
        device,
        sequence,
        kind,
        payload,
        signature,
    })
}

fn take_signature(bytes: &[u8]) -> Result<[u8; 64]> {
    bytes
        .try_into()
        .map_err(|_| Error::Event("signature has the wrong length"))
}
