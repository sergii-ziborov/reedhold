//! One signed reaction record. Weight is computed, not stored as gospel.

use crate::kind::ReactionKind;
use reedhold_core::{ContentId, Digest32, IdentityId};

/// One like / dislike / endorse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reaction {
    /// Who reacted.
    pub author: IdentityId,
    /// Content id. Never plaintext.
    pub target: ContentId,
    /// Like, dislike, or endorse.
    pub kind: ReactionKind,
    /// Sybil-cluster tag. Zeros means independent.
    pub cluster: Digest32,
    /// Topic id. Zeros means global-only.
    pub topic: Digest32,
    /// Unix seconds when the event was signed.
    pub created_at: u64,
}

impl Reaction {
    /// Duplicate key: one kind per author per target.
    #[must_use]
    pub const fn key(self) -> (IdentityId, ContentId, ReactionKind) {
        (self.author, self.target, self.kind)
    }
}
