//! Host API for reputation v0. No tokens.

use reedhold_core::{ContentId, Digest32, IdentityId, Result};
use reedhold_rep::{Graph, IdentityRep, Reaction, ReactionKind, transfer};
use serde::Serialize;

/// Public identity strength snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct IdentityScoreView {
    /// Identity hex.
    pub identity: String,
    /// Folded 0..=10000 strength. Not a balance.
    pub strength: u32,
    /// Remaining influence units this epoch.
    pub budget_left: u32,
}

/// Settled content reputation.
#[derive(Clone, Debug, Serialize)]
pub struct ContentScoreView {
    /// Target content hex.
    pub target: String,
    /// Mature positive weight.
    pub positive: u32,
    /// Mature negative weight.
    pub negative: u32,
    /// Net after dislike lambda.
    pub net: u32,
}

/// One reaction's current weight.
#[derive(Clone, Debug, Serialize)]
pub struct ReactionView {
    /// `like`, `dislike`, or `endorse`.
    pub kind: String,
    /// Milli-weight at `now`.
    pub weight: u32,
}

/// In-process reputation book.
pub struct RepSession {
    graph: Graph,
}

impl RepSession {
    /// Empty book.
    #[must_use]
    pub fn open() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    /// Seed identity dimensions for a simulation.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the hex id is invalid.
    pub fn seed(
        &mut self,
        identity_hex: &str,
        continuity: u32,
        social: u32,
        content: u32,
        curation: u32,
    ) -> Result<IdentityScoreView> {
        let id = IdentityId::from_hex(identity_hex)?;
        self.graph.seed(
            id,
            IdentityRep {
                continuity,
                social,
                content,
                curation,
                contribution: 0,
                moderation: 0,
            },
        );
        self.identity(identity_hex, 0)
    }

    /// Record a reaction. `now` is unix seconds from the host.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Reputation`] on a duplicate or unknown kind.
    pub fn react(
        &mut self,
        author_hex: &str,
        target_hex: &str,
        kind: &str,
        cluster_hex: &str,
        now: u64,
    ) -> Result<ReactionView> {
        let parsed = ReactionKind::from_name(kind)
            .ok_or(reedhold_core::Error::Reputation("unknown reaction kind"))?;
        let reaction = Reaction {
            author: IdentityId::from_hex(author_hex)?,
            target: ContentId::from_digest(Digest32::from_hex(target_hex)?),
            kind: parsed,
            cluster: parse_optional(cluster_hex)?,
            topic: Digest32::from_bytes([0; 32]),
            created_at: now,
        };
        let weight = self.graph.react(reaction)?;
        Ok(ReactionView {
            kind: parsed.as_str().to_owned(),
            weight,
        })
    }

    /// Identity snapshot at `now`.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the hex id is invalid.
    pub fn identity(&self, identity_hex: &str, now: u64) -> Result<IdentityScoreView> {
        let id = IdentityId::from_hex(identity_hex)?;
        let rep = self.graph.identity(id);
        Ok(IdentityScoreView {
            identity: id.to_hex(),
            strength: rep.strength(),
            budget_left: self.graph.budget_left(id, now),
        })
    }

    /// Content snapshot at `now`.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when the hex id is invalid.
    pub fn content(&self, target_hex: &str, now: u64) -> Result<ContentScoreView> {
        let target = ContentId::from_digest(Digest32::from_hex(target_hex)?);
        let score = self.graph.content(target, now);
        Ok(ContentScoreView {
            target: target.as_digest().to_hex(),
            positive: score.positive,
            negative: score.negative,
            net: score.net,
        })
    }

    /// Always fails. Reputation is not transferable.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Reputation`].
    pub fn transfer(from_hex: &str, to_hex: &str, amount: u32) -> Result<()> {
        transfer(
            IdentityId::from_hex(from_hex)?,
            IdentityId::from_hex(to_hex)?,
            amount,
        )
    }
}

fn parse_optional(hex: &str) -> Result<Digest32> {
    if hex.is_empty() {
        return Ok(Digest32::from_bytes([0; 32]));
    }
    Digest32::from_hex(hex)
}

#[cfg(test)]
mod tests {
    use super::RepSession;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn pump_is_cheap_and_transfer_is_impossible() {
        let mut rep = RepSession::open();
        let target = hex(1);
        for n in 2_u8..=30 {
            rep.react(&hex(n), &target, "like", &hex(9), 0).unwrap();
        }
        let instant = rep.content(&target, 0).unwrap().net;
        let mut calm = RepSession::open();
        for n in 2_u8..=6 {
            calm.seed(&hex(n), 5000, 5000, 5000, 5000).unwrap();
            calm.react(&hex(n), &target, "like", "", 0).unwrap();
        }
        let week = calm.content(&target, 7 * 86_400).unwrap().net;
        assert!(instant < week);
        assert!(RepSession::transfer(&hex(2), &hex(3), 10).is_err());
    }
}
