//! Sealed message envelope. Blindplane replaces this AEAD later.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Error, Result};

const ENVELOPE_TAG: u8 = 0x30;
const NONCE_LEN: usize = 24;

/// Encrypted payload. Network nodes store this, not plaintext.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageEnvelope {
    /// XChaCha20-Poly1305 nonce.
    pub nonce: [u8; NONCE_LEN],
    /// Ciphertext including the tag.
    pub ciphertext: Vec<u8>,
}

impl MessageEnvelope {
    /// Canonical encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a field cannot be encoded.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_u8(ENVELOPE_TAG);
        writer.write_bytes(&self.nonce)?;
        writer.write_bytes(&self.ciphertext)?;
        Ok(writer.finish())
    }

    /// Decode a canonical envelope.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the buffer is not an envelope.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        if reader.read_u8()? != ENVELOPE_TAG {
            return Err(Error::Event("unknown envelope tag"));
        }
        let nonce = take_nonce(reader.read_bytes()?)?;
        let ciphertext = reader.read_bytes()?.to_vec();
        reader.finish()?;
        Ok(Self { nonce, ciphertext })
    }
}

/// Seal `plaintext` under a 32-byte conversation key.
///
/// # Errors
///
/// Returns [`Error::Event`] or [`Error::Entropy`] on AEAD failure.
pub fn seal_message(key: &[u8; 32], plaintext: &[u8]) -> Result<MessageEnvelope> {
    let mut nonce = [0_u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce).map_err(|_| Error::Entropy)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|_| Error::Event("envelope seal failed"))?;
    Ok(MessageEnvelope { nonce, ciphertext })
}

/// Open a sealed envelope.
///
/// # Errors
///
/// Returns [`Error::Event`] when the key is wrong or the bytes are corrupt.
pub fn open_message(key: &[u8; 32], envelope: &MessageEnvelope) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(
            XNonce::from_slice(&envelope.nonce),
            envelope.ciphertext.as_ref(),
        )
        .map_err(|_| Error::Event("envelope open failed"))
}

fn take_nonce(bytes: &[u8]) -> Result<[u8; NONCE_LEN]> {
    bytes
        .try_into()
        .map_err(|_| Error::Event("envelope nonce has the wrong length"))
}

#[cfg(test)]
mod tests {
    use super::{open_message, seal_message};

    #[test]
    fn envelope_round_trips() {
        let key = [9_u8; 32];
        let sealed = seal_message(&key, b"secret").unwrap();
        assert_eq!(open_message(&key, &sealed).unwrap(), b"secret");
        assert!(open_message(&[0_u8; 32], &sealed).is_err());
    }
}
