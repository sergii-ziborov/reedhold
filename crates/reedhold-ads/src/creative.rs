//! Signed-style campaign object. Payload is a content id, never pixels.

use reedhold_core::Digest32;

/// One campaign. Hosts store bytes elsewhere; the market only sees the cid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Creative {
    /// Advertiser identity digest.
    pub advertiser: Digest32,
    /// Campaign id.
    pub campaign: Digest32,
    /// Content-addressed creative. Not the ad bytes.
    pub payload: Digest32,
    /// Topic bucket this campaign bids in.
    pub topic: Digest32,
    /// Inclusive lowest audience bucket.
    pub bucket_min: u8,
    /// Inclusive highest audience bucket.
    pub bucket_max: u8,
    /// Remaining sandbox credits.
    pub budget: u64,
    /// Exclusive expiry epoch.
    pub expiry: u64,
    /// Hide/dislike risk, milli. Starts at 1000.
    pub risk_milli: u32,
}

impl Creative {
    /// New campaign at unit risk.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        advertiser: Digest32,
        campaign: Digest32,
        payload: Digest32,
        topic: Digest32,
        bucket_min: u8,
        bucket_max: u8,
        budget: u64,
        expiry: u64,
    ) -> Self {
        Self {
            advertiser,
            campaign,
            payload,
            topic,
            bucket_min,
            bucket_max,
            budget,
            expiry,
            risk_milli: 1000,
        }
    }

    /// Whether this creative may show in `bucket` at `epoch`.
    #[must_use]
    pub fn matches(self, topic: Digest32, bucket: u8, epoch: u64) -> bool {
        self.topic == topic
            && bucket >= self.bucket_min
            && bucket <= self.bucket_max
            && epoch < self.expiry
    }
}
