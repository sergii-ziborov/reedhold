//! Short-lived advertising-operator certificate.

use crate::limits::AdvertisingLimits;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use reedhold_codec::{Reader, Writer};
use reedhold_core::{Error, Result};

const CERT_TAG: u8 = 0x41;

/// Operator may sell/distribute ads until `valid_until`. Nothing else.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdOperatorCertificate {
    /// Genesis advertising public key that issued this cert.
    pub issuer: [u8; 32],
    /// Operator verifying key.
    pub operator: [u8; 32],
    /// Inclusive start epoch.
    pub valid_from: u64,
    /// Exclusive end epoch.
    pub valid_until: u64,
    /// Max campaign budget the operator may commit.
    pub max_budget: u64,
    /// Issuer signature.
    pub signature: [u8; 64],
}

impl AdOperatorCertificate {
    pub(crate) fn issue(
        issuer: &SigningKey,
        issuer_public: [u8; 32],
        operator: [u8; 32],
        valid_from: u64,
        valid_until: u64,
        max_budget: u64,
    ) -> Self {
        let mut cert = Self {
            issuer: issuer_public,
            operator,
            valid_from,
            valid_until,
            max_budget,
            signature: [0_u8; 64],
        };
        let body = encode_body(&cert);
        cert.signature = issuer.sign(&body).to_bytes();
        cert
    }

    /// Capability mask. Always market-only.
    #[must_use]
    pub const fn limits(&self) -> AdvertisingLimits {
        AdvertisingLimits::GENESIS
    }

    /// Encode the signed certificate.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a field cannot be encoded.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_bytes(&encode_body(self))?;
        writer.write_bytes(&self.signature)?;
        Ok(writer.finish())
    }

    /// Decode and verify against the published genesis public key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the certificate is forged or expired
    /// relative to `now_epoch` if `now_epoch >= valid_until`.
    pub fn decode_verify(bytes: &[u8], genesis_public: &[u8; 32], now_epoch: u64) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let body = reader.read_bytes()?.to_vec();
        let signature = take64(reader.read_bytes()?)?;
        reader.finish()?;
        let cert = decode_body(&body, signature)?;
        if &cert.issuer != genesis_public {
            return Err(Error::Event("certificate issuer is not genesis"));
        }
        if now_epoch < cert.valid_from || now_epoch >= cert.valid_until {
            return Err(Error::Event("certificate is outside its validity window"));
        }
        let verifying = VerifyingKey::from_bytes(genesis_public)
            .map_err(|_| Error::Event("invalid genesis public key"))?;
        verifying
            .verify(&body, &Signature::from_bytes(&cert.signature))
            .map_err(|_| Error::Event("certificate signature rejected"))?;
        Ok(cert)
    }
}

fn encode_body(cert: &AdOperatorCertificate) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.write_u8(CERT_TAG);
    writer.write_digest32(&cert.issuer);
    writer.write_digest32(&cert.operator);
    writer.write_u64(cert.valid_from);
    writer.write_u64(cert.valid_until);
    writer.write_u64(cert.max_budget);
    writer.finish()
}

fn decode_body(bytes: &[u8], signature: [u8; 64]) -> Result<AdOperatorCertificate> {
    let mut reader = Reader::new(bytes);
    if reader.read_u8()? != CERT_TAG {
        return Err(Error::Event("unknown ad certificate tag"));
    }
    let issuer = reader.read_digest32()?;
    let operator = reader.read_digest32()?;
    let valid_from = reader.read_u64()?;
    let valid_until = reader.read_u64()?;
    let max_budget = reader.read_u64()?;
    reader.finish()?;
    Ok(AdOperatorCertificate {
        issuer,
        operator,
        valid_from,
        valid_until,
        max_budget,
        signature,
    })
}

fn take64(bytes: &[u8]) -> Result<[u8; 64]> {
    bytes
        .try_into()
        .map_err(|_| Error::Event("certificate signature has the wrong length"))
}

#[cfg(test)]
mod tests {
    use crate::AdvertisingRoot;

    #[test]
    fn certificate_verifies_and_cannot_control_the_mesh() {
        let root = AdvertisingRoot::from_seed(&[5_u8; 32]);
        let cert = root.issue_operator([9_u8; 32], 10, 20, 1_000);
        let encoded = cert.encode().unwrap();
        let checked =
            crate::AdOperatorCertificate::decode_verify(&encoded, &root.public_key(), 15).unwrap();
        assert!(checked.limits().is_market_only());
        assert!(!root.can_sign_user_event());
        assert!(
            crate::AdOperatorCertificate::decode_verify(&encoded, &root.public_key(), 20).is_err()
        );
    }
}
