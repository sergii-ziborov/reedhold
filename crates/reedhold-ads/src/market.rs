//! In-process attention market. Genesis is optional. No user-id targeting.

use crate::auction::{Bid, Clearing, clear};
use crate::bucket::floor;
use crate::creative::Creative;
use crate::distributor::Distributor;
use reedhold_core::{Digest32, Error, Result};
use std::collections::BTreeMap;

const SLOTS: usize = 8;
const HIDE_RISK: u32 = 200;

/// Settlement split of one clearing. Sandbox credits, not real money.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Split {
    /// Attention owner.
    pub user: u64,
    /// Distributors.
    pub distributor: u64,
    /// Protocol treasury.
    pub treasury: u64,
    /// Burned.
    pub burn: u64,
}

/// Attention market. Does not require a genesis operator.
#[derive(Clone, Debug, Default)]
pub struct Market {
    creatives: BTreeMap<Digest32, Creative>,
    distributors: BTreeMap<Digest32, Distributor>,
    books: BTreeMap<(Digest32, u8, u64), Vec<Bid>>,
    cleared: BTreeMap<(Digest32, u8, u64), Clearing>,
}

impl Market {
    /// Empty market. No company key is stored.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Post a campaign. Payload must already be a content id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ads`] when the campaign already exists.
    pub fn post(&mut self, creative: Creative) -> Result<()> {
        if self.creatives.contains_key(&creative.campaign) {
            return Err(Error::Ads("duplicate campaign"));
        }
        self.creatives.insert(creative.campaign, creative);
        Ok(())
    }

    /// Register a distributor. Weak accounts are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ads`] when strength is below the protocol threshold.
    pub fn register(&mut self, distributor: Distributor) -> Result<()> {
        if !Distributor::eligible(distributor.strength) {
            return Err(Error::Ads("distributor strength is below threshold"));
        }
        self.distributors.insert(distributor.id, distributor);
        Ok(())
    }

    /// Sealed bid. Price must clear the bucket floor after creative risk.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ads`] when the campaign is unknown or the bid is too low.
    pub fn bid(
        &mut self,
        advertiser: Digest32,
        campaign: Digest32,
        topic: Digest32,
        bucket: u8,
        epoch: u64,
        price: u64,
    ) -> Result<()> {
        let creative = self
            .creatives
            .get(&campaign)
            .ok_or(Error::Ads("unknown campaign"))?;
        if creative.advertiser != advertiser {
            return Err(Error::Ads("advertiser does not own the campaign"));
        }
        if !creative.matches(topic, bucket, epoch) {
            return Err(Error::Ads("campaign does not match this book"));
        }
        if price < floor(bucket, creative.risk_milli) {
            return Err(Error::Ads("bid is below the bucket floor"));
        }
        if price > creative.budget {
            return Err(Error::Ads("bid exceeds remaining budget"));
        }
        self.books
            .entry((topic, bucket, epoch))
            .or_default()
            .push(Bid {
                advertiser,
                campaign,
                price,
            });
        Ok(())
    }

    /// Batch-clear one book. Works with zero distributors and no genesis key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ads`] when the book is empty.
    pub fn clear(&mut self, topic: Digest32, bucket: u8, epoch: u64) -> Result<Clearing> {
        let bids = self
            .books
            .remove(&(topic, bucket, epoch))
            .ok_or(Error::Ads("no bids in this book"))?;
        let slots = self.slot_count().min(SLOTS);
        let clearing = clear(bids, slots.max(1));
        self.cleared
            .insert((topic, bucket, epoch), clearing.clone());
        Ok(clearing)
    }

    /// Local selector. Topics and bucket only — never a user id.
    #[must_use]
    pub fn select(&self, topic: Digest32, bucket: u8, epoch: u64) -> Option<Digest32> {
        self.cleared
            .get(&(topic, bucket, epoch))
            .and_then(|clearing| clearing.winners.first().copied())
    }

    /// Hide/dislike. Raises future floor; does not halt the market.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ads`] when the campaign is unknown.
    pub fn hide(&mut self, campaign: Digest32) -> Result<u32> {
        let creative = self
            .creatives
            .get_mut(&campaign)
            .ok_or(Error::Ads("unknown campaign"))?;
        creative.risk_milli = creative.risk_milli.saturating_add(HIDE_RISK).min(5000);
        Ok(creative.risk_milli)
    }

    /// Split sandbox credits for a cleared book. Not real money.
    #[must_use]
    pub fn settle(&self, topic: Digest32, bucket: u8, epoch: u64) -> Split {
        let Some(clearing) = self.cleared.get(&(topic, bucket, epoch)) else {
            return Split {
                user: 0,
                distributor: 0,
                treasury: 0,
                burn: 0,
            };
        };
        let sold = u64::try_from(clearing.winners.len()).unwrap_or(0);
        let gross = clearing.price.saturating_mul(sold);
        Split {
            user: gross * 40 / 100,
            distributor: gross * 30 / 100,
            treasury: gross * 20 / 100,
            burn: gross * 10 / 100,
        }
    }

    fn slot_count(&self) -> usize {
        let cap: u32 = self.distributors.values().map(Distributor::capacity).sum();
        usize::try_from(cap.max(1)).unwrap_or(SLOTS).min(SLOTS)
    }
}

#[cfg(test)]
mod tests {
    use super::Market;
    use crate::creative::Creative;
    use crate::distributor::Distributor;
    use reedhold_core::Digest32;

    fn d(byte: u8) -> Digest32 {
        Digest32::from_bytes([byte; 32])
    }

    fn campaign(market: &mut Market, byte: u8, budget: u64) {
        market
            .post(Creative::new(
                d(byte),
                d(byte),
                d(50),
                d(7),
                0,
                5,
                budget,
                10,
            ))
            .unwrap();
    }

    #[test]
    fn market_clears_without_genesis_and_never_takes_a_user_id() {
        let mut market = Market::new();
        campaign(&mut market, 1, 1000);
        campaign(&mut market, 2, 1000);
        market
            .register(Distributor {
                id: d(9),
                strength: 4000,
                quality: 1000,
            })
            .unwrap();
        market.bid(d(1), d(1), d(7), 3, 1, 80).unwrap();
        market.bid(d(2), d(2), d(7), 3, 1, 50).unwrap();
        let clearing = market.clear(d(7), 3, 1).unwrap();
        assert!(!clearing.winners.is_empty());
        assert_eq!(market.select(d(7), 3, 1), Some(clearing.winners[0]));
        assert!(
            market
                .register(Distributor {
                    id: d(8),
                    strength: 100,
                    quality: 1000,
                })
                .is_err()
        );
        let before = crate::bucket::floor(3, 1000);
        let risk = market.hide(d(1)).unwrap();
        assert!(crate::bucket::floor(3, risk) > before);
    }
}
