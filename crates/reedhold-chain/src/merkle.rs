//! Binary Merkle proofs over 32-byte heads. Off-chain; the chain stores the root.

use crate::hash::digest;
use reedhold_core::{Digest32, DomainTag, Error, Result};

/// Inclusion proof for one leaf under a subtree root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleProof {
    /// Leaf index in the original list.
    pub index: u32,
    /// Sibling hashes from leaf to root.
    pub siblings: Vec<Digest32>,
}

/// Merkle root of `leaves`. Empty list is the zero digest.
#[must_use]
pub fn merkle_root(leaves: &[Digest32]) -> Digest32 {
    fold(tagged_leaves(leaves))
}

/// Prove `leaves[index]` is in [`merkle_root`].
///
/// # Errors
///
/// Returns [`Error::Chain`] when `index` is out of range.
pub fn merkle_prove(leaves: &[Digest32], index: usize) -> Result<MerkleProof> {
    if index >= leaves.len() {
        return Err(Error::Chain("leaf index out of range"));
    }
    let mut layer = tagged_leaves(leaves);
    let mut cursor = index;
    let mut siblings = Vec::new();
    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            let last = *layer.last().unwrap_or(&Digest32::from_bytes([0; 32]));
            layer.push(last);
        }
        let pair = cursor ^ 1;
        if let Some(sibling) = layer.get(pair) {
            siblings.push(*sibling);
        }
        cursor /= 2;
        layer = next_layer(&layer);
    }
    let index = u32::try_from(index).map_err(|_| Error::Chain("leaf index out of range"))?;
    Ok(MerkleProof { index, siblings })
}

/// Check a proof against `root`.
#[must_use]
pub fn merkle_verify(leaf: Digest32, proof: &MerkleProof, root: Digest32) -> bool {
    let mut acc = tagged_leaf(&leaf);
    let mut cursor = proof.index;
    for sibling in &proof.siblings {
        acc = if cursor.is_multiple_of(2) {
            tagged_node(&acc, sibling)
        } else {
            tagged_node(sibling, &acc)
        };
        cursor /= 2;
    }
    acc == root
}

fn tagged_leaves(leaves: &[Digest32]) -> Vec<Digest32> {
    leaves.iter().map(tagged_leaf).collect()
}

fn tagged_leaf(leaf: &Digest32) -> Digest32 {
    digest(DomainTag::ChainMerkle, &[&[0_u8], leaf.as_bytes()])
}

fn tagged_node(left: &Digest32, right: &Digest32) -> Digest32 {
    digest(
        DomainTag::ChainMerkle,
        &[&[1_u8], left.as_bytes(), right.as_bytes()],
    )
}

fn next_layer(layer: &[Digest32]) -> Vec<Digest32> {
    layer
        .chunks(2)
        .map(|chunk| tagged_node(&chunk[0], &chunk[1]))
        .collect()
}

fn fold(mut layer: Vec<Digest32>) -> Digest32 {
    if layer.is_empty() {
        return Digest32::from_bytes([0; 32]);
    }
    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            let last = *layer.last().unwrap_or(&Digest32::from_bytes([0; 32]));
            layer.push(last);
        }
        layer = next_layer(&layer);
    }
    layer[0]
}

#[cfg(test)]
mod tests {
    use super::{merkle_prove, merkle_root, merkle_verify};
    use reedhold_core::Digest32;

    #[test]
    fn proof_verifies_and_rejects_a_stranger() {
        let leaves: Vec<Digest32> = (1_u8..=5)
            .map(|byte| Digest32::from_bytes([byte; 32]))
            .collect();
        let root = merkle_root(&leaves);
        let proof = merkle_prove(&leaves, 3).unwrap();
        assert!(merkle_verify(leaves[3], &proof, root));
        assert!(!merkle_verify(leaves[0], &proof, root));
        assert!(merkle_prove(&leaves, 9).is_err());
    }
}
