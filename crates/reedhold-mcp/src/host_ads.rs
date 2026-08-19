//! Attention-market methods on the MCP host.

use crate::host::Host;
use reedhold_api::{ClearingView, MarketSession, SplitView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn ads_open(&mut self) {
        self.ads = Some(MarketSession::open());
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ads_post(
        &mut self,
        advertiser: &str,
        campaign: &str,
        payload: &str,
        topic: &str,
        bucket_min: u8,
        bucket_max: u8,
        budget: u64,
        expiry: u64,
    ) -> Result<()> {
        self.ads_mut()?.post(
            advertiser, campaign, payload, topic, bucket_min, bucket_max, budget, expiry,
        )
    }

    pub(crate) fn ads_register(&mut self, id: &str, strength: u32) -> Result<()> {
        self.ads_mut()?.register(id, strength)
    }

    pub(crate) fn ads_bid(
        &mut self,
        advertiser: &str,
        campaign: &str,
        topic: &str,
        bucket: u8,
        epoch: u64,
        price: u64,
    ) -> Result<()> {
        self.ads_mut()?
            .bid(advertiser, campaign, topic, bucket, epoch, price)
    }

    pub(crate) fn ads_clear(
        &mut self,
        topic: &str,
        bucket: u8,
        epoch: u64,
    ) -> Result<ClearingView> {
        self.ads_mut()?.clear(topic, bucket, epoch)
    }

    pub(crate) fn ads_select(&self, topic: &str, bucket: u8, epoch: u64) -> Result<Option<String>> {
        self.ads()?.select(topic, bucket, epoch)
    }

    pub(crate) fn ads_hide(&mut self, campaign: &str) -> Result<u32> {
        self.ads_mut()?.hide(campaign)
    }

    pub(crate) fn ads_settle(&self, topic: &str, bucket: u8, epoch: u64) -> Result<SplitView> {
        self.ads()?.settle(topic, bucket, epoch)
    }

    fn ads(&self) -> Result<&MarketSession> {
        self.ads.as_ref().ok_or(Error::Ads("ad market is not open"))
    }

    fn ads_mut(&mut self) -> Result<&mut MarketSession> {
        self.ads.as_mut().ok_or(Error::Ads("ad market is not open"))
    }
}
