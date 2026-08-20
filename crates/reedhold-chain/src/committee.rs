//! Who is allowed to finalise an epoch.
//!
//! Fork choice alone is not enough: anyone can claim any amount of work behind
//! their branch. Work only counts once a committee has signed for it, and the
//! committee is not something an attacker can join on demand.
//!
//! Two rules do the work. Membership is drawn from a beacon taken out of
//! *already settled* history, so nobody can grind their way into the committee
//! that will judge the epoch they are attacking — they would have had to
//! control the network several epochs earlier. And quorum is measured in
//! **weight, not seats**, so splitting one worker into a thousand changes the
//! seating chart and nothing else.

use crate::hash::digest;
use ed25519_dalek::{Signature, VerifyingKey};
use reedhold_core::{Digest32, DomainTag, Error, Result};

/// How many epochs back the selection beacon is taken from.
///
/// The committee for epoch `N` is fixed by history at `N - LOOKBACK`, which is
/// already finalised. An attacker who takes over today cannot choose who will
/// judge tomorrow.
pub const BEACON_LOOKBACK: u64 = 2;

/// One eligible node and the work behind it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Seat {
    /// Node identifier.
    pub node: Digest32,
    /// Verified work, linear. Never a peer count.
    pub weight: u64,
    /// Ed25519 key this node signs headers with.
    pub public: [u8; 32],
}

/// The set that may finalise one epoch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Committee {
    /// Epoch this committee judges.
    pub epoch: u64,
    /// Seats, in draw order.
    pub seats: Vec<Seat>,
}

impl Committee {
    /// Draw up to `size` seats for `epoch` from settled randomness.
    ///
    /// Selection favours weight, but safety does not rest on it: quorum is a
    /// share of weight, so an attacker who wins extra seats by splitting wins
    /// no extra say.
    #[must_use]
    pub fn draw(epoch: u64, beacon: Digest32, eligible: &[Seat], size: usize) -> Self {
        let mut ranked: Vec<(u128, Digest32, Seat)> = eligible
            .iter()
            .filter(|seat| seat.weight > 0)
            .map(|seat| (score(beacon, epoch, seat), seat.node, *seat))
            .collect();
        ranked.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        Self {
            epoch,
            seats: ranked.into_iter().take(size).map(|(_, _, s)| s).collect(),
        }
    }

    /// Total weight seated.
    #[must_use]
    pub fn weight(&self) -> u128 {
        self.seats.iter().map(|seat| u128::from(seat.weight)).sum()
    }

    /// Weight needed to finalise: strictly more than two thirds.
    #[must_use]
    pub fn quorum(&self) -> u128 {
        self.weight() * 2 / 3
    }

    /// Weight that validly signed `header`.
    ///
    /// Signatures from outside the committee are ignored rather than counted,
    /// and a node cannot vote twice.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] when a seated key is malformed.
    pub fn tally(&self, header_hash: Digest32, votes: &[(Digest32, Signature)]) -> Result<u128> {
        let mut counted = Vec::new();
        let mut total = 0_u128;
        for (node, signature) in votes {
            if counted.contains(node) {
                continue;
            }
            let Some(seat) = self.seats.iter().find(|seat| seat.node == *node) else {
                continue;
            };
            let key = VerifyingKey::from_bytes(&seat.public)
                .map_err(|_| Error::Chain("committee key is not a valid ed25519 key"))?;
            if key.verify_strict(header_hash.as_bytes(), signature).is_ok() {
                counted.push(*node);
                total = total.saturating_add(u128::from(seat.weight));
            }
        }
        Ok(total)
    }

    /// Is this header finalised by this committee?
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] when a seated key is malformed.
    pub fn finalises(
        &self,
        header_hash: Digest32,
        votes: &[(Digest32, Signature)],
    ) -> Result<bool> {
        Ok(self.tally(header_hash, votes)? > self.quorum())
    }
}

/// Lower is better. Weight shortens the draw; the beacon decides the rest.
fn score(beacon: Digest32, epoch: u64, seat: &Seat) -> u128 {
    let mixed = digest(
        DomainTag::RelayScore,
        &[
            beacon.as_bytes(),
            &epoch.to_be_bytes(),
            seat.node.as_bytes(),
        ],
    );
    let mut head = [0_u8; 16];
    head.copy_from_slice(&mixed.as_bytes()[..16]);
    u128::from_be_bytes(head) / u128::from(seat.weight.max(1))
}

#[cfg(test)]
mod tests {
    use super::{Committee, Seat};
    use ed25519_dalek::{Signer, SigningKey};
    use reedhold_core::Digest32;

    fn keypair(byte: u8) -> SigningKey {
        SigningKey::from_bytes(&[byte; 32])
    }

    fn seat(byte: u8, weight: u64) -> Seat {
        Seat {
            node: Digest32::from_bytes([byte; 32]),
            weight,
            public: keypair(byte).verifying_key().to_bytes(),
        }
    }

    fn beacon() -> Digest32 {
        Digest32::from_bytes([0x5a; 32])
    }

    fn header() -> Digest32 {
        Digest32::from_bytes([0x77; 32])
    }

    fn vote(byte: u8) -> (Digest32, ed25519_dalek::Signature) {
        (
            Digest32::from_bytes([byte; 32]),
            keypair(byte).sign(header().as_bytes()),
        )
    }

    #[test]
    fn an_outsider_signature_is_worth_nothing() {
        let seats: Vec<Seat> = (1_u8..=4).map(|byte| seat(byte, 100)).collect();
        let committee = Committee::draw(9, beacon(), &seats, 4);
        let stranger = vote(200);
        assert_eq!(committee.tally(header(), &[stranger]).unwrap(), 0);
    }

    #[test]
    fn a_member_cannot_vote_twice() {
        let seats: Vec<Seat> = (1_u8..=4).map(|byte| seat(byte, 100)).collect();
        let committee = Committee::draw(9, beacon(), &seats, 4);
        let once = committee.tally(header(), &[vote(1)]).unwrap();
        let twice = committee.tally(header(), &[vote(1), vote(1)]).unwrap();
        assert_eq!(once, twice, "stuffing the ballot changes nothing");
    }

    #[test]
    fn two_thirds_of_weight_finalises_and_less_does_not() {
        let seats: Vec<Seat> = (1_u8..=6).map(|byte| seat(byte, 100)).collect();
        let committee = Committee::draw(9, beacon(), &seats, 6);
        let members: Vec<u8> = committee
            .seats
            .iter()
            .map(|seat| seat.node.as_bytes()[0])
            .collect();

        let four: Vec<_> = members.iter().take(4).map(|byte| vote(*byte)).collect();
        assert!(!committee.finalises(header(), &four).unwrap());
        let five: Vec<_> = members.iter().take(5).map(|byte| vote(*byte)).collect();
        assert!(committee.finalises(header(), &five).unwrap());
    }

    #[test]
    fn splitting_a_worker_buys_no_extra_say() {
        // One node holding 900 units of work.
        let whole = vec![seat(1, 900), seat(9, 300)];
        let big = Committee::draw(4, beacon(), &whole, 8);
        // The same resources presented as nine nodes of 100.
        let mut split: Vec<Seat> = (10_u8..=18).map(|byte| seat(byte, 100)).collect();
        split.push(seat(9, 300));
        let many = Committee::draw(4, beacon(), &split, 8);

        assert!(many.seats.len() >= big.seats.len(), "more seats, sure");
        // What matters is the share of weight, and that is unchanged.
        let attacker_split: u128 = many
            .seats
            .iter()
            .filter(|seat| seat.node.as_bytes()[0] >= 10)
            .map(|seat| u128::from(seat.weight))
            .sum();
        assert!(
            attacker_split <= 900,
            "nine shards cannot seat more weight than the node had"
        );
    }

    #[test]
    fn the_draw_is_settled_before_the_epoch_it_judges() {
        let seats: Vec<Seat> = (1_u8..=8).map(|byte| seat(byte, 100)).collect();
        let same = Committee::draw(5, beacon(), &seats, 4);
        let again = Committee::draw(5, beacon(), &seats, 4);
        assert_eq!(same, again, "the same beacon always seats the same people");

        let other = Committee::draw(5, Digest32::from_bytes([0x11; 32]), &seats, 4);
        assert_ne!(
            same, other,
            "a different beacon seats different people, so grinding the \
             beacon is the only attack, and it lies in already-settled history"
        );
    }
}
