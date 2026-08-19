//! Host API for the advertising sandbox. No user-id targeting.

use reedhold_ads::{Creative, Distributor, Market, bucket as strength_bucket};
use reedhold_core::{Digest32, Result};
use serde::Serialize;

/// One batch clearing.
#[derive(Clone, Debug, Serialize)]
pub struct ClearingView {
    /// Winning campaign hexes, highest bid first.
    pub winners: Vec<String>,
    /// Uniform price every winner pays.
    pub price: u64,
}

/// Sandbox credit split. Not real money.
#[derive(Clone, Debug, Serialize)]
pub struct SplitView {
    /// Attention owner.
    pub user: u64,
    /// Distributors.
    pub distributor: u64,
    /// Protocol treasury.
    pub treasury: u64,
    /// Burned.
    pub burn: u64,
}

/// In-process attention market.
pub struct MarketSession {
    market: Market,
}

impl MarketSession {
    /// Empty market. Genesis is not required.
    #[must_use]
    pub fn open() -> Self {
        Self {
            market: Market::new(),
        }
    }

    /// Audience bucket for a strength score. Hosts never pass a user id here.
    #[must_use]
    pub fn bucket(strength: u32) -> u8 {
        strength_bucket(strength)
    }

    /// Post a campaign. `payload` is a content id hex, not ad bytes.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when a hex field is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn post(
        &mut self,
        advertiser_hex: &str,
        campaign_hex: &str,
        payload_hex: &str,
        topic_hex: &str,
        bucket_min: u8,
        bucket_max: u8,
        budget: u64,
        expiry: u64,
    ) -> Result<()> {
        self.market.post(Creative::new(
            Digest32::from_hex(advertiser_hex)?,
            Digest32::from_hex(campaign_hex)?,
            Digest32::from_hex(payload_hex)?,
            Digest32::from_hex(topic_hex)?,
            bucket_min,
            bucket_max,
            budget,
            expiry,
        ))
    }

    /// Register a distributor by strength. Weak accounts are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Ads`] below the protocol threshold.
    pub fn register(&mut self, id_hex: &str, strength: u32) -> Result<()> {
        self.market.register(Distributor {
            id: Digest32::from_hex(id_hex)?,
            strength,
            quality: 1000,
        })
    }

    /// Sealed bid for one `(topic, bucket, epoch)` book.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Ads`] when the bid is below the floor.
    pub fn bid(
        &mut self,
        advertiser_hex: &str,
        campaign_hex: &str,
        topic_hex: &str,
        bucket: u8,
        epoch: u64,
        price: u64,
    ) -> Result<()> {
        self.market.bid(
            Digest32::from_hex(advertiser_hex)?,
            Digest32::from_hex(campaign_hex)?,
            Digest32::from_hex(topic_hex)?,
            bucket,
            epoch,
            price,
        )
    }

    /// Clear the book. Works with no genesis operator.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Ads`] when the book is empty.
    pub fn clear(&mut self, topic_hex: &str, bucket: u8, epoch: u64) -> Result<ClearingView> {
        let clearing = self
            .market
            .clear(Digest32::from_hex(topic_hex)?, bucket, epoch)?;
        Ok(ClearingView {
            winners: clearing.winners.iter().map(|id| id.to_hex()).collect(),
            price: clearing.price,
        })
    }

    /// Pick a cleared campaign. Arguments are topic + bucket, never a user id.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when topic hex is invalid.
    pub fn select(&self, topic_hex: &str, bucket: u8, epoch: u64) -> Result<Option<String>> {
        Ok(self
            .market
            .select(Digest32::from_hex(topic_hex)?, bucket, epoch)
            .map(Digest32::to_hex))
    }

    /// Hide a campaign. Raises its future floor.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Ads`] when the campaign is unknown.
    pub fn hide(&mut self, campaign_hex: &str) -> Result<u32> {
        self.market.hide(Digest32::from_hex(campaign_hex)?)
    }

    /// Split sandbox credits for a cleared book.
    ///
    /// # Errors
    ///
    /// Returns [`reedhold_core::Error::Codec`] when topic hex is invalid.
    pub fn settle(&self, topic_hex: &str, bucket: u8, epoch: u64) -> Result<SplitView> {
        let split = self
            .market
            .settle(Digest32::from_hex(topic_hex)?, bucket, epoch);
        Ok(SplitView {
            user: split.user,
            distributor: split.distributor,
            treasury: split.treasury,
            burn: split.burn,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MarketSession;

    fn hex(byte: u8) -> String {
        reedhold_core::encode_hex(&[byte; 32])
    }

    #[test]
    fn stronger_inventory_costs_more_and_select_has_no_user_id() {
        let mut market = MarketSession::open();
        assert!(MarketSession::bucket(9000) > MarketSession::bucket(100));
        market
            .post(&hex(1), &hex(1), &hex(50), &hex(7), 0, 5, 500, 9)
            .unwrap();
        market.register(&hex(9), 5000).unwrap();
        market.bid(&hex(1), &hex(1), &hex(7), 5, 1, 200).unwrap();
        let clearing = market.clear(&hex(7), 5, 1).unwrap();
        assert_eq!(market.select(&hex(7), 5, 1).unwrap(), Some(hex(1)));
        let split = market.settle(&hex(7), 5, 1).unwrap();
        assert_eq!(
            split.user + split.distributor + split.treasury + split.burn,
            clearing.price
        );
        assert!(market.register(&hex(2), 50).is_err());
    }
}
