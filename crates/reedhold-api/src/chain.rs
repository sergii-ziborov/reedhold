//! Host API over compact chain headers. No message bytes.

use reedhold_chain::{
    EpochRoots, Header, Ledger, LightClient, MerkleProof, merkle_prove, merkle_root, merkle_verify,
};
use reedhold_core::{Digest32, NetworkId, Result, decode_hex};
use serde::Serialize;

/// Hex snapshot of one header. `encoded_len` is independent of social payload.
#[derive(Clone, Debug, Serialize)]
pub struct HeaderView {
    /// Height. Genesis is 0.
    pub height: u64,
    /// Social epoch.
    pub epoch: u64,
    /// Previous header hash hex.
    pub prev: String,
    /// This header hash hex.
    pub hash: String,
    /// Combined state root hex.
    pub state_root: String,
    /// Identity subtree hex.
    pub identity: String,
    /// Group subtree hex.
    pub groups: String,
    /// Storage subtree hex.
    pub storage: String,
    /// Canonical encoded size in bytes.
    pub encoded_len: u32,
}

/// Merkle inclusion proof for one 32-byte head.
#[derive(Clone, Debug, Serialize)]
pub struct ProofView {
    /// Subtree root hex.
    pub root: String,
    /// Leaf index.
    pub index: u32,
    /// Sibling hashes, hex.
    pub siblings: Vec<String>,
}

/// In-process compact chain plus a bounded light-client window.
pub struct ChainSession {
    ledger: Ledger,
    light: LightClient,
}

impl ChainSession {
    /// Genesis header. Light client starts there.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Chain`] if genesis cannot be followed.
    pub fn open() -> Result<Self> {
        let ledger = Ledger::genesis(NetworkId::DEV);
        let mut light = LightClient::new();
        light.follow(ledger.head())?;
        Ok(Self { ledger, light })
    }

    /// Commit 32-byte subtree roots. Empty hex is the zero digest.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when a hex root is invalid.
    pub fn commit(
        &mut self,
        epoch: u64,
        identity_hex: &str,
        groups_hex: &str,
        storage_hex: &str,
    ) -> Result<HeaderView> {
        let roots = EpochRoots {
            identity: parse_root(identity_hex)?,
            groups: parse_root(groups_hex)?,
            storage: parse_root(storage_hex)?,
            reputation: Digest32::from_bytes([0; 32]),
            ads: Digest32::from_bytes([0; 32]),
        };
        let header = self.ledger.commit(epoch, roots)?;
        self.light.follow(header)?;
        Ok(header_view(&header))
    }

    /// Latest header the light client follows.
    #[must_use]
    pub fn head(&self) -> HeaderView {
        header_view(&self.light.head().unwrap_or_else(|| self.ledger.head()))
    }

    /// Bounded window. Never the full historical payload.
    #[must_use]
    pub fn headers(&self) -> Vec<HeaderView> {
        self.light.window().iter().map(header_view).collect()
    }

    /// Ingest an encoded header hex into the light client.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Chain`] on rollback or a broken link.
    pub fn follow(&mut self, header_hex: &str) -> Result<HeaderView> {
        let header = Header::decode(&decode_hex(header_hex)?)?;
        self.light.follow(header)?;
        Ok(header_view(&header))
    }

    /// Prove `leaves[index]` against the Merkle root of `leaves`.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Chain`] when the index is out of range.
    pub fn prove(&self, leaf_hexes: &[String], index: u32) -> Result<ProofView> {
        let leaves = parse_leaves(leaf_hexes)?;
        let idx = usize::try_from(index).unwrap_or(usize::MAX);
        let proof = merkle_prove(&leaves, idx)?;
        Ok(proof_view(merkle_root(&leaves), &proof))
    }

    /// Verify a proof. Does not look at private message bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when a hex field is invalid.
    pub fn verify(
        &self,
        leaf_hex: &str,
        root_hex: &str,
        index: u32,
        sibling_hexes: &[String],
    ) -> Result<bool> {
        let proof = MerkleProof {
            index,
            siblings: parse_leaves(sibling_hexes)?,
        };
        Ok(merkle_verify(
            parse_root(leaf_hex)?,
            &proof,
            parse_root(root_hex)?,
        ))
    }
}

fn parse_root(hex: &str) -> Result<Digest32> {
    if hex.is_empty() {
        return Ok(Digest32::from_bytes([0; 32]));
    }
    Digest32::from_hex(hex)
}

fn parse_leaves(hexes: &[String]) -> Result<Vec<Digest32>> {
    let mut out = Vec::with_capacity(hexes.len());
    for hex in hexes {
        out.push(parse_root(hex)?);
    }
    Ok(out)
}

fn header_view(header: &Header) -> HeaderView {
    let encoded = header.encode();
    HeaderView {
        height: header.height,
        epoch: header.epoch,
        prev: header.prev.to_hex(),
        hash: header.hash().to_hex(),
        state_root: header.state_root().to_hex(),
        identity: header.roots.identity.to_hex(),
        groups: header.roots.groups.to_hex(),
        storage: header.roots.storage.to_hex(),
        encoded_len: u32::try_from(encoded.len()).unwrap_or(u32::MAX),
    }
}

fn proof_view(root: Digest32, proof: &MerkleProof) -> ProofView {
    ProofView {
        root: root.to_hex(),
        index: proof.index,
        siblings: proof
            .siblings
            .iter()
            .map(|digest| digest.to_hex())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::ChainSession;
    use reedhold_chain::HEADER_WINDOW;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn headers_stay_compact_and_the_window_is_bounded() {
        let mut chain = ChainSession::open().unwrap();
        let first = chain.commit(1, &hex(1), &hex(2), &hex(3)).unwrap();
        let second = chain.commit(2, &hex(9), "", "").unwrap();
        assert_eq!(first.encoded_len, second.encoded_len);
        assert_ne!(first.state_root, second.state_root);
        for epoch in 3..=HEADER_WINDOW + 10 {
            chain
                .commit(u64::try_from(epoch).unwrap_or(0), &hex(1), "", "")
                .unwrap();
        }
        assert_eq!(chain.headers().len(), HEADER_WINDOW);
        let leaves = vec![hex(1), hex(2), hex(3)];
        let proof = chain.prove(&leaves, 1).unwrap();
        assert!(
            chain
                .verify(&hex(2), &proof.root, proof.index, &proof.siblings)
                .unwrap()
        );
        assert!(
            !chain
                .verify(&hex(1), &proof.root, proof.index, &proof.siblings)
                .unwrap()
        );
    }
}
