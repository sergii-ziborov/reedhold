//! Committing every balance to one hash.
//!
//! A balance that lives only in a node's memory is a number somebody can
//! retype. Folded into a Merkle root and written into the header, it stops
//! being editable: changing one account changes the root, which changes the
//! header hash, which breaks the `prev` link of every header after it. The
//! tampered chain no longer descends from genesis, so the network does not
//! weigh it against the honest one — it refuses it as a different network.

use crate::merkle::{MerkleProof, merkle_prove, merkle_root, merkle_verify};
use reedhold_core::{Digest32, DomainTag, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// One account and what it holds.
#[must_use]
pub fn balance_leaf(account: Digest32, credits: u64) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update(DomainTag::ChainMerkle.as_bytes());
    hasher.update(b"balance");
    hasher.update(account.as_bytes());
    hasher.update(credits.to_be_bytes());
    Digest32::from_bytes(hasher.finalize().into())
}

/// Root over every balance, in account order so the result is canonical.
#[must_use]
pub fn balances_root(balances: &BTreeMap<Digest32, u64>) -> Digest32 {
    let leaves: Vec<Digest32> = balances
        .iter()
        .map(|(account, credits)| balance_leaf(*account, *credits))
        .collect();
    merkle_root(&leaves)
}

/// Prove one account's balance against [`balances_root`].
///
/// # Errors
///
/// Returns [`reedhold_core::Error::Chain`] when the account is absent.
pub fn prove_balance(balances: &BTreeMap<Digest32, u64>, account: Digest32) -> Result<MerkleProof> {
    let leaves: Vec<Digest32> = balances
        .iter()
        .map(|(id, credits)| balance_leaf(*id, *credits))
        .collect();
    let index = balances
        .keys()
        .position(|id| *id == account)
        .ok_or(reedhold_core::Error::Chain("no such account"))?;
    merkle_prove(&leaves, index)
}

/// Check a balance claim against a committed root.
#[must_use]
pub fn verify_balance(
    account: Digest32,
    credits: u64,
    proof: &MerkleProof,
    root: Digest32,
) -> bool {
    merkle_verify(balance_leaf(account, credits), proof, root)
}

#[cfg(test)]
mod tests {
    use super::{balances_root, prove_balance, verify_balance};
    use crate::fork::{Branch, ForkChoice};
    use crate::header::Header;
    use crate::roots::EpochRoots;
    use reedhold_core::{Digest32, NetworkId};
    use std::collections::BTreeMap;

    fn account(byte: u8) -> Digest32 {
        Digest32::from_bytes([byte; 32])
    }

    fn book() -> BTreeMap<Digest32, u64> {
        [(account(1), 100), (account(2), 250), (account(3), 7)]
            .into_iter()
            .collect()
    }

    #[test]
    fn a_single_credit_changes_the_whole_root() {
        let honest = balances_root(&book());
        let mut greedy = book();
        greedy.insert(account(3), 8);
        assert_ne!(honest, balances_root(&greedy), "one unit must be visible");
    }

    #[test]
    fn an_account_can_prove_what_it_holds() {
        let balances = book();
        let root = balances_root(&balances);
        let proof = prove_balance(&balances, account(2)).unwrap();
        assert!(verify_balance(account(2), 250, &proof, root));
        assert!(
            !verify_balance(account(2), 251, &proof, root),
            "a claim of more than was committed must fail"
        );
        assert!(prove_balance(&balances, account(9)).is_err());
    }

    #[test]
    fn editing_a_balance_detaches_the_chain_from_genesis() {
        let genesis = Header::genesis(NetworkId::DEV);
        let mut honest_roots = EpochRoots::empty();
        honest_roots.ledger = balances_root(&book());
        let honest_head = genesis.successor(1, honest_roots);
        let honest = Branch {
            headers: vec![genesis, honest_head],
            work: vec![10, 10],
        };

        // The attacker awards themselves credits and rebuilds the header.
        let mut cooked = book();
        cooked.insert(account(1), 1_000_000);
        let mut cooked_roots = EpochRoots::empty();
        cooked_roots.ledger = balances_root(&cooked);
        let cooked_head = genesis.successor(1, cooked_roots);
        assert_ne!(cooked_head.hash(), honest_head.hash());

        // Anything built on top of the forged header no longer links to the
        // honest history, and a node anchored at genesis simply refuses it.
        let mut orphan = Branch {
            headers: vec![genesis, cooked_head, honest_head.successor(2, honest_roots)],
            work: vec![10, 10, u64::from(u32::MAX)],
        };
        let rule = ForkChoice::new(genesis.hash());
        assert!(
            rule.accepts(&honest, &orphan).is_err(),
            "a rewritten balance breaks every link after it"
        );

        // Even a clean, far heavier branch cannot reopen a settled height.
        orphan.headers = vec![genesis, cooked_head];
        orphan.work = vec![10, u64::from(u32::MAX)];
        assert!(
            rule.finalised_to(1).accepts(&honest, &orphan).is_err(),
            "past the checkpoint, buying weight buys nothing"
        );
    }
}
