//! Reed-Solomon shards. k of n reconstruct the object.

use crate::tier::DurabilityTier;
use reed_solomon_erasure::galois_8::ReedSolomon;
use reedhold_core::{Error, Result};

/// Coding parameters for one durability class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coding {
    /// Shards required to reconstruct.
    pub k: u8,
    /// Total shards placed.
    pub n: u8,
}

/// One encoded shard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shard {
    /// Index in `0..n`.
    pub index: u8,
    /// Shard bytes, equal length across the set.
    pub bytes: Vec<u8>,
}

impl Coding {
    /// Default parameters. `n - k` is at least a third of `n`.
    #[must_use]
    pub const fn for_tier(tier: DurabilityTier) -> Self {
        match tier {
            DurabilityTier::CriticalIdentity | DurabilityTier::PersonalHistory => {
                Self { k: 4, n: 6 }
            }
            DurabilityTier::PublicSocial => Self { k: 3, n: 5 },
            DurabilityTier::LargeMedia => Self { k: 2, n: 3 },
        }
    }

    /// How many shards can be lost.
    #[must_use]
    pub const fn parity(self) -> u8 {
        self.n.saturating_sub(self.k)
    }

    /// Encode `data` into `n` shards. First four bytes of the pad carry length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] when Reed-Solomon rejects the parameters.
    pub fn encode(self, data: &[u8]) -> Result<Vec<Shard>> {
        let codec = rs(self)?;
        let mut pieces = split_padded(data, usize::from(self.k));
        let shard_len = pieces[0].len();
        for _ in 0..usize::from(self.parity()) {
            pieces.push(vec![0_u8; shard_len]);
        }
        codec
            .encode(&mut pieces)
            .map_err(|_| Error::Storage("erasure encode failed"))?;
        Ok(pieces
            .into_iter()
            .enumerate()
            .map(|(index, bytes)| Shard {
                index: u8::try_from(index).unwrap_or(0),
                bytes,
            })
            .collect())
    }

    /// Reconstruct original bytes from any `k` live shards.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] when fewer than `k` shards remain.
    pub fn decode(self, present: &[Option<Shard>]) -> Result<Vec<u8>> {
        if present.iter().flatten().count() < usize::from(self.k) {
            return Err(Error::Storage("not enough live shards"));
        }
        let codec = rs(self)?;
        let mut slots: Vec<Option<Vec<u8>>> = present
            .iter()
            .map(|shard| shard.as_ref().map(|item| item.bytes.clone()))
            .collect();
        if slots.len() < usize::from(self.n) {
            slots.resize(usize::from(self.n), None);
        }
        codec
            .reconstruct(&mut slots)
            .map_err(|_| Error::Storage("erasure reconstruct failed"))?;
        let mut padded = Vec::new();
        for slot in slots.into_iter().take(usize::from(self.k)) {
            padded.extend(slot.ok_or(Error::Storage("reconstruct left a hole"))?);
        }
        unpad(&padded)
    }
}

fn rs(coding: Coding) -> Result<ReedSolomon> {
    ReedSolomon::new(usize::from(coding.k), usize::from(coding.parity()))
        .map_err(|_| Error::Storage("invalid reed-solomon parameters"))
}

fn split_padded(data: &[u8], k: usize) -> Vec<Vec<u8>> {
    let mut buf = u32::try_from(data.len())
        .unwrap_or(u32::MAX)
        .to_le_bytes()
        .to_vec();
    buf.extend_from_slice(data);
    let shard_len = buf.len().div_ceil(k).max(1);
    buf.resize(shard_len * k, 0);
    buf.chunks(shard_len).map(<[u8]>::to_vec).collect()
}

fn unpad(padded: &[u8]) -> Result<Vec<u8>> {
    if padded.len() < 4 {
        return Err(Error::Storage("padded object is truncated"));
    }
    let len = u32::from_le_bytes([padded[0], padded[1], padded[2], padded[3]]) as usize;
    let body = padded
        .get(4..)
        .ok_or(Error::Storage("padded object is truncated"))?;
    if body.len() < len {
        return Err(Error::Storage("padded object is truncated"));
    }
    Ok(body[..len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::Coding;

    #[test]
    fn survives_two_of_six_missing() {
        let coding = Coding { k: 4, n: 6 };
        let shards = coding.encode(b"identity-manifest").unwrap();
        let mut present: Vec<Option<_>> = shards.into_iter().map(Some).collect();
        present[1] = None;
        present[4] = None;
        assert_eq!(coding.decode(&present).unwrap(), b"identity-manifest");
    }
}
