//! In-process durable grid. Contracted shards, not cache.

use crate::budget::NodeBudget;
use crate::code::{Coding, Shard};
use crate::holder::HolderId;
use crate::place::assign;
use crate::tier::DurabilityTier;
use reedhold_core::{Digest32, Error, Result};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Metadata for one stored object.
#[derive(Clone, Debug)]
pub struct ObjectMeta {
    /// Content id of the original bytes.
    pub id: Digest32,
    /// Durability class.
    pub tier: DurabilityTier,
    /// Coding used.
    pub coding: Coding,
    /// Holder for each shard index, if still placed.
    pub holders: Vec<Option<HolderId>>,
}

/// Durable shard grid with hard per-holder quotas.
#[derive(Clone, Debug)]
pub struct Grid {
    budget: NodeBudget,
    company: Option<HolderId>,
    alive: BTreeSet<HolderId>,
    used: BTreeMap<HolderId, u64>,
    shards: BTreeMap<(Digest32, u8), (HolderId, Shard)>,
    objects: BTreeMap<Digest32, ObjectMeta>,
}

impl Grid {
    /// Empty grid. `holders` are the eligible failure domains.
    #[must_use]
    pub fn new(budget: NodeBudget, holders: &[HolderId], company: Option<HolderId>) -> Self {
        Self {
            budget,
            company,
            alive: holders.iter().copied().collect(),
            used: BTreeMap::new(),
            shards: BTreeMap::new(),
            objects: BTreeMap::new(),
        }
    }

    /// Encode and place `data`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] on quota or placement failure.
    pub fn put(&mut self, data: &[u8], tier: DurabilityTier) -> Result<ObjectMeta> {
        let id = object_id(data);
        let coding = Coding::for_tier(tier);
        let encoded = coding.encode(data)?;
        let live: Vec<HolderId> = self.alive.iter().copied().collect();
        let chosen = assign(id, usize::from(coding.n), &live, self.company)?;
        let mut holders = vec![None; usize::from(coding.n)];
        for (shard, holder) in encoded.into_iter().zip(chosen) {
            self.place_shard(id, holder, shard, &mut holders)?;
        }
        let meta = ObjectMeta {
            id,
            tier,
            coding,
            holders,
        };
        self.objects.insert(id, meta.clone());
        Ok(meta)
    }

    /// Reconstruct an object from any `k` live shards.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] when too many shards are gone.
    pub fn get(&self, id: Digest32) -> Result<Vec<u8>> {
        let meta = self
            .objects
            .get(&id)
            .ok_or(Error::Storage("unknown durable object"))?;
        let present = self.present_shards(meta);
        meta.coding.decode(&present)
    }

    /// Drop every shard on `holder`. A third of the grid may disappear.
    pub fn kill(&mut self, holder: HolderId) {
        self.alive.remove(&holder);
        self.used.remove(&holder);
        self.shards.retain(|_, (owner, _)| *owner != holder);
        for meta in self.objects.values_mut() {
            for slot in &mut meta.holders {
                if *slot == Some(holder) {
                    *slot = None;
                }
            }
        }
    }

    /// Rebuild missing shards onto surviving holders.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] when reconstruction or placement fails.
    pub fn repair(&mut self, id: Digest32) -> Result<ObjectMeta> {
        let data = self.get(id)?;
        let meta = self
            .objects
            .get(&id)
            .ok_or(Error::Storage("unknown durable object"))?
            .clone();
        let encoded = meta.coding.encode(&data)?;
        let occupied: BTreeSet<HolderId> = meta.holders.iter().flatten().copied().collect();
        let live: Vec<HolderId> = self
            .alive
            .iter()
            .copied()
            .filter(|holder| !occupied.contains(holder))
            .collect();
        let mut holders = meta.holders;
        let missing: Vec<Shard> = encoded
            .into_iter()
            .filter(|shard| {
                holders
                    .get(usize::from(shard.index))
                    .copied()
                    .flatten()
                    .is_none()
            })
            .collect();
        let replacements = assign(id, missing.len(), &live, self.company)?;
        for (shard, holder) in missing.into_iter().zip(replacements) {
            self.place_shard(id, holder, shard, &mut holders)?;
        }
        let updated = ObjectMeta { holders, ..meta };
        self.objects.insert(id, updated.clone());
        Ok(updated)
    }

    /// Bytes currently contracted on `holder`.
    #[must_use]
    pub fn used(&self, holder: HolderId) -> u64 {
        self.used.get(&holder).copied().unwrap_or(0)
    }

    fn present_shards(&self, meta: &ObjectMeta) -> Vec<Option<Shard>> {
        (0..meta.coding.n)
            .map(|index| {
                self.shards
                    .get(&(meta.id, index))
                    .map(|(_, shard)| shard.clone())
            })
            .collect()
    }

    fn place_shard(
        &mut self,
        id: Digest32,
        holder: HolderId,
        shard: Shard,
        holders: &mut [Option<HolderId>],
    ) -> Result<()> {
        let add = u64::try_from(shard.bytes.len()).unwrap_or(u64::MAX);
        let next = self.used(holder).saturating_add(add);
        if !self.budget.allows_durable(next) {
            return Err(Error::Storage("holder durable quota exceeded"));
        }
        let index = shard.index;
        self.shards.insert((id, index), (holder, shard));
        self.used.insert(holder, next);
        if let Some(slot) = holders.get_mut(usize::from(index)) {
            *slot = Some(holder);
        }
        Ok(())
    }
}

fn object_id(data: &[u8]) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(b"reedhold/object/v1");
    hasher.update(data);
    Digest32::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::Grid;
    use crate::budget::NodeBudget;
    use crate::holder::HolderId;
    use crate::tier::DurabilityTier;
    use reedhold_core::Digest32;

    fn holder(byte: u8) -> HolderId {
        HolderId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    #[test]
    fn third_loss_then_repair() {
        let holders: Vec<HolderId> = (1_u8..=9).map(holder).collect();
        let mut grid = Grid::new(NodeBudget::PHONE_LIGHT, &holders, Some(holder(99)));
        let meta = grid
            .put(b"recovery-manifest", DurabilityTier::CriticalIdentity)
            .unwrap();
        let first: Vec<HolderId> = meta.holders.iter().flatten().copied().take(2).collect();
        for dead in &first {
            grid.kill(*dead);
        }
        assert_eq!(grid.get(meta.id).unwrap(), b"recovery-manifest");
        let repaired = grid.repair(meta.id).unwrap();
        let next: Vec<HolderId> = repaired
            .holders
            .iter()
            .flatten()
            .copied()
            .filter(|holder| !first.contains(holder))
            .take(2)
            .collect();
        for dead in &next {
            grid.kill(*dead);
        }
        assert_eq!(grid.get(meta.id).unwrap(), b"recovery-manifest");
    }
}
