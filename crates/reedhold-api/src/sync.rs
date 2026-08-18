//! Host-facing sync roster. No company host is required.

use reedhold_core::{Error, NetworkId, Result, decode32};
use reedhold_mesh::{DEFAULT_RELAY_COUNT, SyncEpoch, SyncPlan};
use serde::Serialize;

/// Snapshot of today's transitional hosts.
#[derive(Clone, Debug, Serialize)]
pub struct SyncPlanView {
    /// Epoch index (Unix day).
    pub epoch: u64,
    /// Hex peer ids selected as relays for this epoch.
    pub relays: Vec<String>,
    /// Optional company accelerator. Never required.
    pub company: Option<String>,
    /// Always false. The site host is not a protocol root.
    pub company_required: bool,
    /// Always false. Blocking today's relays does not stop the mesh.
    pub blocking_is_fatal: bool,
}

/// Draw the epoch roster from candidate peer hex ids.
///
/// `prior_commit_hex` is 32 bytes. Later this is the previous chain head.
///
/// # Errors
///
/// Returns [`Error::Codec`] when a hex field is the wrong length.
pub fn sync_plan(
    epoch: u64,
    prior_commit_hex: &str,
    candidate_hexes: &[String],
    company_hex: Option<&str>,
    relay_count: Option<u16>,
) -> Result<SyncPlanView> {
    let prior = decode32(prior_commit_hex)?;
    let mut candidates = Vec::with_capacity(candidate_hexes.len());
    for hex in candidate_hexes {
        candidates.push(reedhold_mesh::PeerId::from_hex(hex)?);
    }
    let company = match company_hex {
        Some(hex) if !hex.is_empty() => Some(reedhold_mesh::PeerId::from_hex(hex)?),
        _ => None,
    };
    let default_count = u16::try_from(DEFAULT_RELAY_COUNT).unwrap_or(8);
    let limit = usize::from(relay_count.unwrap_or(default_count));
    if limit == 0 {
        return Err(Error::Codec("relay_count must be at least 1"));
    }
    let plan = SyncPlan::draw(
        NetworkId::DEV,
        SyncEpoch { index: epoch },
        &prior,
        &candidates,
        company,
        limit,
    );
    Ok(SyncPlanView {
        epoch: plan.epoch.index,
        relays: plan.relays.iter().map(|peer| peer.to_hex()).collect(),
        company: plan.company.map(reedhold_mesh::PeerId::to_hex),
        company_required: plan.requires_company(),
        blocking_is_fatal: plan.blocking_is_fatal(),
    })
}

#[cfg(test)]
mod tests {
    use super::sync_plan;

    #[test]
    fn company_block_is_irrelevant() {
        let candidates: Vec<String> = (1_u8..=16)
            .map(|byte| reedhold_core::encode_hex(&[byte; 32]))
            .collect();
        let company = reedhold_core::encode_hex(&[99_u8; 32]);
        let plan = sync_plan(5, &"00".repeat(32), &candidates, Some(&company), Some(4)).unwrap();
        assert!(!plan.company_required);
        assert!(!plan.blocking_is_fatal);
        assert_eq!(plan.relays.len(), 4);
        assert!(!plan.relays.contains(&company));
    }
}
