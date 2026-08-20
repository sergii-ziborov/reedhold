//! Epoch emission.
//!
//! A per-node ceiling bounds nobody: total supply becomes `nodes x cap`, so
//! the currency inflates exactly as fast as the network grows. A fixed global
//! budget per epoch, split in proportion to proven work, makes more workers
//! mean less per worker for the same effort — competition instead of
//! inflation, and a supply curve anyone can predict.

use std::collections::BTreeMap;

/// Credits minted per epoch, for the whole network.
pub const EPOCH_MINT_BUDGET: u64 = 1_000_000;

/// Work the network must prove in an epoch to earn the full budget.
///
/// A young network with one node cannot be allowed to mint a full epoch every
/// six hours: a month of that quietly dwarfs any declared allocation, and each
/// step looks legitimate. The obvious guard — a ceiling per node — is a trap.
/// It only bites when an honest node's fair share is large, so it pays that
/// node to split into several, which is the very behaviour the whole design
/// exists to make pointless.
///
/// Scaling the budget by *total* work has neither problem. Splitting changes
/// no totals, so it changes nothing; and a weak network mints proportionally
/// little because it did proportionally little.
pub const TARGET_EPOCH_WORK: u64 = 100_000;

/// Credits this epoch actually prints, given the work behind it.
#[must_use]
pub fn epoch_budget(total_work: u64) -> u64 {
    let scaled = u128::from(EPOCH_MINT_BUDGET).saturating_mul(u128::from(total_work))
        / u128::from(TARGET_EPOCH_WORK);
    u64::try_from(scaled)
        .unwrap_or(u64::MAX)
        .min(EPOCH_MINT_BUDGET)
}

/// Split the epoch's budget across claims, proportional to work.
#[must_use]
pub fn settle<K: Ord + Copy>(claims: &BTreeMap<K, u64>) -> BTreeMap<K, u64> {
    let total: u64 = claims.values().copied().sum();
    if total == 0 {
        return BTreeMap::new();
    }
    let budget = epoch_budget(total);
    claims
        .iter()
        .map(|(node, claim)| {
            let share = u128::from(*claim).saturating_mul(u128::from(budget)) / u128::from(total);
            (*node, u64::try_from(share).unwrap_or(u64::MAX))
        })
        .collect()
}

/// What the network actually printed for an epoch.
#[must_use]
pub fn minted_total<K: Ord + Copy>(settled: &BTreeMap<K, u64>) -> u64 {
    settled.values().copied().sum()
}

#[cfg(test)]
mod tests {
    use super::{EPOCH_MINT_BUDGET, TARGET_EPOCH_WORK, epoch_budget, minted_total, settle};
    use std::collections::BTreeMap;

    fn claims(entries: &[(u8, u64)]) -> BTreeMap<u8, u64> {
        entries.iter().copied().collect()
    }

    #[test]
    fn supply_is_capped_however_much_work_arrives() {
        assert_eq!(epoch_budget(TARGET_EPOCH_WORK), EPOCH_MINT_BUDGET);
        assert_eq!(epoch_budget(TARGET_EPOCH_WORK * 1000), EPOCH_MINT_BUDGET);
        assert!(epoch_budget(TARGET_EPOCH_WORK / 4) < EPOCH_MINT_BUDGET);
    }

    #[test]
    fn more_workers_at_the_same_total_work_means_less_each() {
        let few = settle(&claims(&[(1, 500), (2, 500)]));
        let many: BTreeMap<u8, u64> = (1_u8..=50).map(|node| (node, 20_u64)).collect();
        let many = settle(&many);
        assert_eq!(minted_total(&few), minted_total(&many));
        assert!(many[&1] < few[&1], "the pie is split, not enlarged");
    }

    #[test]
    fn the_same_work_is_paid_the_same() {
        let settled = settle(&claims(&[(1, 500), (2, 500), (3, 1000)]));
        assert_eq!(settled[&1], settled[&2]);
        assert!(settled[&3] > settled[&1], "twice the work, twice the pay");
    }

    #[test]
    fn a_lone_node_on_an_empty_network_mints_almost_nothing() {
        let settled = settle(&claims(&[(1, 1_000)]));
        assert!(
            settled[&1] * 50 < EPOCH_MINT_BUDGET,
            "one percent of the target work must not print a full epoch"
        );
    }

    #[test]
    fn splitting_into_many_nodes_earns_nothing_extra() {
        let whole = settle(&claims(&[(1, 30_000), (9, 70_000)]));
        let split = settle(&claims(&[
            (1, 10_000),
            (2, 10_000),
            (3, 10_000),
            (9, 70_000),
        ]));
        let attacker: u64 = [1_u8, 2, 3].iter().filter_map(|n| split.get(n)).sum();
        assert_eq!(
            whole[&1], attacker,
            "three shards of one node take the same slice as the node did"
        );
        assert_eq!(minted_total(&whole), minted_total(&split));
    }

    #[test]
    fn an_idle_epoch_prints_nothing() {
        assert!(settle(&claims(&[])).is_empty());
        assert_eq!(minted_total(&settle(&claims(&[(1, 0)]))), 0);
    }
}
