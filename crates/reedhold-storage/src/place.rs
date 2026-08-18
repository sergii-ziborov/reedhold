//! Deterministic shard placement. Company is never a required holder.

use crate::holder::HolderId;
use reedhold_core::{Digest32, Error, Result};
use sha2::{Digest, Sha256};

/// Assign `count` distinct live holders for one object.
///
/// # Errors
///
/// Returns [`Error::Storage`] when there are not enough independent holders.
pub fn assign(
    object: Digest32,
    count: usize,
    holders: &[HolderId],
    company: Option<HolderId>,
) -> Result<Vec<HolderId>> {
    let pool: Vec<HolderId> = holders
        .iter()
        .copied()
        .filter(|holder| company != Some(*holder))
        .collect();
    if pool.len() < count {
        return Err(Error::Storage("not enough independent holders"));
    }
    let mut ranked: Vec<([u8; 32], HolderId)> = pool
        .into_iter()
        .map(|holder| (score(object, holder), holder))
        .collect();
    ranked.sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    ranked.dedup_by_key(|entry| entry.1);
    Ok(ranked
        .into_iter()
        .take(count)
        .map(|(_, holder)| holder)
        .collect())
}

fn score(object: Digest32, holder: HolderId) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"reedhold/place/v1");
    hasher.update(object.as_bytes());
    hasher.update(holder.as_digest().as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::assign;
    use crate::holder::HolderId;
    use reedhold_core::Digest32;

    fn holder(byte: u8) -> HolderId {
        HolderId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn company_is_never_assigned() {
        let holders: Vec<HolderId> = (1_u8..=8).map(holder).collect();
        let company = holder(99);
        let mut pool = holders.clone();
        pool.push(company);
        let placed = assign(Digest32::from_bytes([3; 32]), 6, &pool, Some(company)).unwrap();
        assert_eq!(placed.len(), 6);
        assert!(!placed.contains(&company));
    }
}
