//! Batch uniform-price auction. All winners pay the same clearing price.

use reedhold_core::Digest32;

/// One sealed bid for `(topic, bucket, epoch)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bid {
    /// Advertiser.
    pub advertiser: Digest32,
    /// Campaign.
    pub campaign: Digest32,
    /// Bid price in sandbox credits.
    pub price: u64,
}

/// Result of one batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clearing {
    /// Winning campaigns, highest bid first.
    pub winners: Vec<Digest32>,
    /// Uniform price every winner pays.
    pub price: u64,
}

/// Uniform price: first rejected bid if the book is full, else the lowest winner.
#[must_use]
pub fn clear(mut bids: Vec<Bid>, slots: usize) -> Clearing {
    if slots == 0 || bids.is_empty() {
        return Clearing {
            winners: Vec::new(),
            price: 0,
        };
    }
    bids.sort_by(|left, right| right.price.cmp(&left.price));
    let take = slots.min(bids.len());
    let price = if bids.len() > take {
        bids[take].price
    } else {
        bids[take - 1].price
    };
    let winners = bids
        .into_iter()
        .take(take)
        .map(|bid| bid.campaign)
        .collect();
    Clearing { winners, price }
}

#[cfg(test)]
mod tests {
    use super::{Bid, clear};
    use reedhold_core::Digest32;

    fn bid(byte: u8, price: u64) -> Bid {
        Bid {
            advertiser: Digest32::from_bytes([byte; 32]),
            campaign: Digest32::from_bytes([byte; 32]),
            price,
        }
    }

    #[test]
    fn winners_pay_the_first_rejected_price() {
        let clearing = clear(vec![bid(1, 100), bid(2, 80), bid(3, 40)], 2);
        assert_eq!(clearing.winners.len(), 2);
        assert_eq!(clearing.price, 40);
        assert_eq!(clearing.winners[0], Digest32::from_bytes([1; 32]));
    }
}
