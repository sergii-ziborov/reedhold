//! In-process reputation graph. Simulation first, no money.

use crate::factor::{
    budget_factor, curator_factor, epoch_budget, independence, rep_factor, topic_factor,
};
use crate::identity::IdentityRep;
use crate::kind::ReactionKind;
use crate::maturity::maturity;
use crate::milli::{Milli, mul};
use crate::reaction::Reaction;
use reedhold_core::{ContentId, Digest32, Error, IdentityId, Result};
use std::collections::{BTreeMap, BTreeSet};

const WEEK: u64 = 7 * 86_400;
const DISLIKE_LAMBDA: Milli = 1200;

/// Settled scores for one object.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContentScore {
    /// Mature positive weight.
    pub positive: u32,
    /// Mature negative weight.
    pub negative: u32,
    /// `positive - 1.2 * negative`, saturating.
    pub net: u32,
}

/// Live reputation book.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    identities: BTreeMap<IdentityId, IdentityRep>,
    reactions: Vec<Reaction>,
    seen: BTreeSet<(IdentityId, ContentId, ReactionKind)>,
    clusters: BTreeMap<(ContentId, Digest32), u32>,
    spent: BTreeMap<(IdentityId, u64), u32>,
}

impl Graph {
    /// Empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed or overwrite an identity's dimensions.
    pub fn seed(&mut self, id: IdentityId, rep: IdentityRep) {
        self.identities.insert(id, rep);
    }

    /// Identity snapshot. Missing identities are zeros.
    #[must_use]
    pub fn identity(&self, id: IdentityId) -> IdentityRep {
        self.identities.get(&id).copied().unwrap_or_default()
    }

    /// Remaining influence units this epoch.
    #[must_use]
    pub fn budget_left(&self, id: IdentityId, now: u64) -> u32 {
        let cap = epoch_budget(self.identity(id).strength());
        cap.saturating_sub(self.spent_in(id, now))
    }

    /// Record a reaction and return its current weight at `now`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Reputation`] on an unknown kind or a duplicate.
    pub fn react(&mut self, reaction: Reaction) -> Result<Milli> {
        if !self.seen.insert(reaction.key()) {
            return Err(Error::Reputation("duplicate reaction"));
        }
        let prior = if reaction.cluster == Digest32::from_bytes([0; 32]) {
            0
        } else {
            self.clusters
                .get(&(reaction.target, reaction.cluster))
                .copied()
                .unwrap_or(0)
        };
        let spent = self.spent_in(reaction.author, reaction.created_at);
        let weight = self.weight(&reaction, reaction.created_at, prior, spent);
        if reaction.cluster != Digest32::from_bytes([0; 32]) {
            let count = self
                .clusters
                .entry((reaction.target, reaction.cluster))
                .or_insert(0);
            *count = count.saturating_add(1);
        }
        let epoch = reaction.created_at / WEEK;
        let spent = self.spent.entry((reaction.author, epoch)).or_insert(0);
        *spent = spent.saturating_add(reaction.kind.cost());
        self.reactions.push(reaction);
        Ok(weight)
    }

    /// Content score at `now`. Old junk does not gain from age alone.
    #[must_use]
    pub fn content(&self, target: ContentId, now: u64) -> ContentScore {
        let mut positive = 0_u32;
        let mut negative = 0_u32;
        for reaction in &self.reactions {
            if reaction.target != target {
                continue;
            }
            let prior = cluster_prior(self, reaction);
            let spent = self.spent_in(reaction.author, reaction.created_at);
            let weight = self.weight(reaction, now, prior, spent);
            if reaction.kind.sign() < 0 {
                negative = negative.saturating_add(weight);
            } else {
                positive = positive.saturating_add(weight);
            }
        }
        let penalty = mul(negative, DISLIKE_LAMBDA);
        ContentScore {
            positive,
            negative,
            net: positive.saturating_sub(penalty),
        }
    }

    fn spent_in(&self, id: IdentityId, now: u64) -> u32 {
        self.spent.get(&(id, now / WEEK)).copied().unwrap_or(0)
    }

    fn weight(&self, reaction: &Reaction, now: u64, prior: u32, spent: u32) -> Milli {
        let identity = self.identity(reaction.author);
        let cap = epoch_budget(identity.strength());
        let age = now.saturating_sub(reaction.created_at);
        let topic_set = reaction.topic != Digest32::from_bytes([0; 32]);
        let mut weight = reaction.kind.base();
        weight = mul(weight, rep_factor(identity.strength()));
        weight = mul(weight, topic_factor(&identity, topic_set));
        weight = mul(weight, maturity(age, reaction.kind));
        weight = mul(weight, independence(prior));
        weight = mul(weight, curator_factor(&identity));
        mul(weight, budget_factor(spent, cap))
    }
}

fn cluster_prior(graph: &Graph, reaction: &Reaction) -> u32 {
    if reaction.cluster == Digest32::from_bytes([0; 32]) {
        0
    } else {
        graph
            .clusters
            .get(&(reaction.target, reaction.cluster))
            .copied()
            .unwrap_or(0)
            .saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;
    use crate::identity::IdentityRep;
    use crate::kind::ReactionKind;
    use crate::reaction::Reaction;
    use reedhold_core::{ContentId, Digest32, IdentityId};

    fn id(byte: u8) -> IdentityId {
        IdentityId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    fn content(byte: u8) -> ContentId {
        ContentId::from_digest(Digest32::from_bytes([byte; 32]))
    }

    fn react(author: u8, cluster: u8, at: u64) -> Reaction {
        Reaction {
            author: id(author),
            target: content(1),
            kind: ReactionKind::Like,
            cluster: Digest32::from_bytes([cluster; 32]),
            topic: Digest32::from_bytes([0; 32]),
            created_at: at,
        }
    }

    #[test]
    fn instant_cluster_pump_is_cheaper_than_mature_independents() {
        let mut pump = Graph::new();
        let mut calm = Graph::new();
        let mature = IdentityRep {
            continuity: 4000,
            social: 4000,
            content: 4000,
            curation: 4000,
            contribution: 4000,
            moderation: 4000,
        };
        for n in 2_u8..=21 {
            calm.seed(id(n), mature);
            calm.react(react(n, n, 0)).unwrap();
        }
        for n in 2_u8..=21 {
            pump.react(react(n, 9, 0)).unwrap();
        }
        let week = 7 * 86_400;
        let cheap = pump.content(content(1), 0).net;
        let settled = calm.content(content(1), week).net;
        assert!(cheap < settled);
        assert!(pump.react(react(2, 9, 1)).is_err());
    }
}
