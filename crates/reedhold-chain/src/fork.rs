//! Which chain is the real one.
//!
//! Three attacks decide the shape of this module.
//!
//! An attacker builds a chain claiming to start earlier. Nothing can start
//! earlier than genesis: a chain whose first header is not the genesis this
//! network was born with is not a fork, it is a different network, and is
//! rejected outright rather than compared.
//!
//! An attacker presents a longer chain as canonical. Length is free to
//! manufacture, so it is never the test. The test is accumulated proven work,
//! and past a finalised checkpoint no amount of work reopens the question.
//!
//! An attacker runs ten thousand zombie machines. Head count buys nothing:
//! weight is linear in verified work, so a botnet must out-work the honest
//! network rather than out-number it. Splitting one worker into a thousand
//! yields exactly the weight of one worker.

use crate::header::Header;
use reedhold_core::{Digest32, Error, Result};

/// A candidate chain, with the work behind it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Branch {
    /// Headers, oldest first. Must start at genesis.
    pub headers: Vec<Header>,
    /// Verified work backing each header, in the same order.
    ///
    /// Linear byte-hours, never a peer count and never a concave curve: both
    /// would pay an attacker to reshape the same resources.
    pub work: Vec<u64>,
}

impl Branch {
    /// Total work behind this branch.
    #[must_use]
    pub fn weight(&self) -> u128 {
        self.work.iter().map(|unit| u128::from(*unit)).sum()
    }

    /// Height of the last header, or zero when empty.
    #[must_use]
    pub fn height(&self) -> u64 {
        self.headers.last().map_or(0, |header| header.height)
    }

    /// Check the internal links and that it descends from `genesis`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] when the branch is empty, does not start at
    /// the expected genesis, or a `prev` link is broken.
    pub fn verify(&self, genesis: Digest32) -> Result<()> {
        let Some(first) = self.headers.first() else {
            return Err(Error::Chain("branch is empty"));
        };
        if self.work.len() != self.headers.len() {
            return Err(Error::Chain("work does not cover every header"));
        }
        if first.height != 0 || first.hash() != genesis {
            return Err(Error::Chain("branch does not start at this genesis"));
        }
        for pair in self.headers.windows(2) {
            let (parent, child) = (&pair[0], &pair[1]);
            if child.height != parent.height.saturating_add(1) || child.prev != parent.hash() {
                return Err(Error::Chain("broken header chain"));
            }
        }
        Ok(())
    }
}

/// The rule a node applies when two chains claim to be the network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkChoice {
    /// Hash of the header this network was born with.
    pub genesis: Digest32,
    /// Height at or below which history is settled and cannot be replaced.
    pub finalised: u64,
}

impl ForkChoice {
    /// Anchor a node to its genesis with nothing yet finalised.
    #[must_use]
    pub const fn new(genesis: Digest32) -> Self {
        Self {
            genesis,
            finalised: 0,
        }
    }

    /// Mark everything up to `height` settled.
    #[must_use]
    pub const fn finalised_to(self, height: u64) -> Self {
        Self {
            genesis: self.genesis,
            finalised: height,
        }
    }

    /// Whether `candidate` may replace `current`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Chain`] when the candidate is malformed, belongs to
    /// another genesis, or tries to rewrite settled history.
    pub fn accepts(&self, current: &Branch, candidate: &Branch) -> Result<bool> {
        candidate.verify(self.genesis)?;
        if candidate.height() < self.finalised {
            return Err(Error::Chain("candidate does not reach the finalised point"));
        }
        if !agrees_up_to(current, candidate, self.finalised) {
            return Err(Error::Chain("candidate rewrites finalised history"));
        }
        Ok(candidate.weight() > current.weight())
    }
}

/// Do both branches carry the same headers up to `height`?
fn agrees_up_to(left: &Branch, right: &Branch, height: u64) -> bool {
    for header in &left.headers {
        if header.height > height {
            break;
        }
        let Some(mirror) = right
            .headers
            .iter()
            .find(|other| other.height == header.height)
        else {
            return false;
        };
        if mirror.hash() != header.hash() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{Branch, ForkChoice};
    use crate::header::Header;
    use crate::roots::EpochRoots;
    use reedhold_core::{Digest32, NetworkId};

    fn roots(byte: u8) -> EpochRoots {
        let mut out = EpochRoots::empty();
        out.identity = Digest32::from_bytes([byte; 32]);
        out
    }

    fn chain(marks: &[u8], work: u64) -> Branch {
        let mut headers = vec![Header::genesis(NetworkId::DEV)];
        for (index, mark) in marks.iter().enumerate() {
            let epoch = u64::try_from(index).unwrap_or(0) + 1;
            let next = headers.last().map_or_else(
                || Header::genesis(NetworkId::DEV),
                |head| head.successor(epoch, roots(*mark)),
            );
            headers.push(next);
        }
        let work = vec![work; headers.len()];
        Branch { headers, work }
    }

    fn choice() -> ForkChoice {
        ForkChoice::new(Header::genesis(NetworkId::DEV).hash())
    }

    #[test]
    fn a_chain_claiming_another_beginning_is_not_a_fork() {
        let honest = chain(&[1, 2, 3], 10);
        let mut forged = chain(&[9, 9, 9, 9, 9, 9], 10);
        // Backdated: the attacker replaces the first header with their own.
        forged.headers[0] = Header::genesis(NetworkId::DEV).successor(99, roots(9));
        let verdict = choice().accepts(&honest, &forged);
        assert!(
            verdict.is_err(),
            "a different genesis is a different network"
        );
    }

    #[test]
    fn a_longer_chain_with_less_work_loses() {
        let honest = chain(&[1, 2], 1_000);
        let padded = chain(&[9, 9, 9, 9, 9, 9, 9, 9], 1);
        assert!(padded.height() > honest.height());
        assert!(
            !choice().accepts(&honest, &padded).unwrap(),
            "height is free to manufacture; work is not"
        );
    }

    #[test]
    fn settled_history_cannot_be_rewritten_at_any_price() {
        let honest = chain(&[1, 2, 3, 4], 10);
        let rich = chain(&[7, 7, 7, 7, 7, 7], u64::from(u32::MAX));
        let rule = choice().finalised_to(2);
        assert!(
            choice().accepts(&honest, &rich).unwrap(),
            "without finality the heavier chain simply wins"
        );
        assert!(
            rule.accepts(&honest, &rich).is_err(),
            "past the checkpoint, weight stops being an argument"
        );
    }

    #[test]
    fn a_botnet_must_out_work_the_network_not_out_number_it() {
        // Ten thousand zombies, each contributing a token amount.
        let botnet_units: u64 = 10_000;
        let botnet = chain(&[9, 9, 9], botnet_units);
        // One honest cohort doing more real byte-hours in total.
        let honest = chain(&[1, 2, 3], botnet_units * 2);
        assert!(
            !choice().accepts(&honest, &botnet).unwrap(),
            "counting machines is not the test"
        );
    }

    #[test]
    fn a_branch_with_a_broken_link_is_refused() {
        let honest = chain(&[1], 10);
        let mut tampered = chain(&[1, 2], 10);
        tampered.headers[2].prev = Digest32::from_bytes([0; 32]);
        assert!(choice().accepts(&honest, &tampered).is_err());
    }
}
