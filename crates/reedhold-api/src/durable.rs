//! Host API for the durable shard grid.

use reedhold_core::{Digest32, Result};
use reedhold_storage::{DurabilityTier, Grid, HolderId, NodeBudget};
use serde::Serialize;

/// Snapshot of a stored object.
#[derive(Clone, Debug, Serialize)]
pub struct ObjectView {
    /// Content id hex.
    pub id: String,
    /// Durability class name.
    pub tier: String,
    /// Reed-Solomon `k`.
    pub k: u8,
    /// Reed-Solomon `n`.
    pub n: u8,
    /// Live holders, one per shard index. Missing shards are empty strings.
    pub holders: Vec<String>,
}

/// Durable grid driven by hex holder ids.
pub struct DurableSession {
    grid: Grid,
}

impl DurableSession {
    /// Open a grid. `company_hex` is never a required holder.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when a hex id is invalid.
    pub fn open(holder_hexes: &[String], company_hex: Option<&str>) -> Result<Self> {
        let mut holders = Vec::with_capacity(holder_hexes.len());
        for hex in holder_hexes {
            holders.push(HolderId::from_hex(hex)?);
        }
        let company = match company_hex {
            Some(hex) if !hex.is_empty() => Some(HolderId::from_hex(hex)?),
            _ => None,
        };
        Ok(Self {
            grid: Grid::new(NodeBudget::PHONE_LIGHT, &holders, company),
        })
    }

    /// Store bytes. Critical/personal tiers use 4-of-6 coding.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Storage`] on quota or placement failure.
    pub fn put(&mut self, payload: &str, tier: &str) -> Result<ObjectView> {
        let meta = self.grid.put(payload.as_bytes(), parse_tier(tier))?;
        Ok(object_view(&meta))
    }

    /// Reconstruct original bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Storage`] when too many shards are gone.
    pub fn get(&self, id_hex: &str) -> Result<String> {
        let bytes = self.grid.get(Digest32::from_hex(id_hex)?)?;
        String::from_utf8(bytes).map_err(|_| reedhold_core::Error::Storage("object is not utf-8"))
    }

    /// Drop a holder and all of its contracted shards.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the hex id is invalid.
    pub fn kill(&mut self, holder_hex: &str) -> Result<()> {
        self.grid.kill(HolderId::from_hex(holder_hex)?);
        Ok(())
    }

    /// Rebuild missing shards onto surviving holders.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Storage`] when reconstruction fails.
    pub fn repair(&mut self, id_hex: &str) -> Result<ObjectView> {
        let meta = self.grid.repair(Digest32::from_hex(id_hex)?)?;
        Ok(object_view(&meta))
    }
}

fn parse_tier(name: &str) -> DurabilityTier {
    match name {
        "personal" => DurabilityTier::PersonalHistory,
        "public" => DurabilityTier::PublicSocial,
        "media" => DurabilityTier::LargeMedia,
        _ => DurabilityTier::CriticalIdentity,
    }
}

fn object_view(meta: &reedhold_storage::ObjectMeta) -> ObjectView {
    ObjectView {
        id: meta.id.to_hex(),
        tier: match meta.tier {
            DurabilityTier::CriticalIdentity => "critical",
            DurabilityTier::PersonalHistory => "personal",
            DurabilityTier::PublicSocial => "public",
            DurabilityTier::LargeMedia => "media",
        }
        .to_owned(),
        k: meta.coding.k,
        n: meta.coding.n,
        holders: meta
            .holders
            .iter()
            .map(|holder| holder.map_or_else(String::new, HolderId::to_hex))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::DurableSession;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn kill_two_holders_then_read() {
        let holders: Vec<String> = (1_u8..=8).map(hex).collect();
        let mut grid = DurableSession::open(&holders, Some(&hex(99))).unwrap();
        let stored = grid.put("hello-durable", "critical").unwrap();
        let live: Vec<String> = stored
            .holders
            .iter()
            .filter(|holder| !holder.is_empty())
            .cloned()
            .collect();
        grid.kill(&live[0]).unwrap();
        grid.kill(&live[1]).unwrap();
        assert_eq!(grid.get(&stored.id).unwrap(), "hello-durable");
    }
}
