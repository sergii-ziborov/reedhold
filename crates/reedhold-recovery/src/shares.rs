//! k-of-n shares of a `MasterSeed`. One share is useless.

use crate::field::{eval, interpolate_zero};
use reedhold_core::{Error, Result};
use reedhold_identity::MasterSeed;

/// One Shamir share. Safe to give to a friend or a second device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeedShare {
    /// X coordinate in `1..=255`.
    pub index: u8,
    /// Shared bytes of the 32-byte seed.
    pub body: [u8; 32],
}

/// Split `seed` into `total` shares, `threshold` of which restore it.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when `threshold`/`total` are invalid, or
/// [`Error::Entropy`] when the OS RNG fails.
pub fn split_seed(seed: &MasterSeed, threshold: u8, total: u8) -> Result<Vec<SeedShare>> {
    if threshold < 2 || total < threshold || total > 16 {
        return Err(Error::Recovery("need 2 <= threshold <= total <= 16"));
    }
    let secret = seed.as_bytes();
    let mut shares = vec![
        SeedShare {
            index: 0,
            body: [0_u8; 32],
        };
        usize::from(total)
    ];
    for (offset, share) in shares.iter_mut().enumerate() {
        let index =
            u8::try_from(offset + 1).map_err(|_| Error::Recovery("share index overflow"))?;
        share.index = index;
    }
    for (byte_index, secret_byte) in secret.iter().enumerate() {
        let coeffs = random_coeffs(*secret_byte, threshold)?;
        for share in &mut shares {
            share.body[byte_index] = eval(&coeffs, share.index);
        }
    }
    Ok(shares)
}

/// Combine at least `threshold` shares back into the seed.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when shares are too few, duplicated, or invalid.
pub fn combine_seed(shares: &[SeedShare], threshold: u8) -> Result<MasterSeed> {
    if threshold < 2 || shares.len() < usize::from(threshold) {
        return Err(Error::Recovery("not enough shares"));
    }
    let used = &shares[..usize::from(threshold)];
    let mut seen = [false; 256];
    for share in used {
        if share.index == 0 || seen[usize::from(share.index)] {
            return Err(Error::Recovery("duplicate or zero share index"));
        }
        seen[usize::from(share.index)] = true;
    }
    let mut secret = [0_u8; 32];
    for (byte_index, secret_byte) in secret.iter_mut().enumerate() {
        let mut points = [(0_u8, 0_u8); 16];
        for (slot, share) in used.iter().enumerate() {
            points[slot] = (share.index, share.body[byte_index]);
        }
        *secret_byte = interpolate_zero(&points[..used.len()])
            .ok_or(Error::Recovery("share interpolation failed"))?;
    }
    Ok(MasterSeed::from_bytes(secret))
}

fn random_coeffs(secret: u8, threshold: u8) -> Result<Vec<u8>> {
    let degree = usize::from(threshold);
    let mut coeffs = vec![0_u8; degree];
    coeffs[0] = secret;
    getrandom::getrandom(&mut coeffs[1..]).map_err(|_| Error::Entropy)?;
    Ok(coeffs)
}

#[cfg(test)]
mod tests {
    use super::{combine_seed, split_seed};
    use reedhold_identity::MasterSeed;

    #[test]
    fn two_of_three_restores_and_one_does_not() {
        let seed = MasterSeed::from_bytes([0x5a_u8; 32]);
        let shares = split_seed(&seed, 2, 3).unwrap();
        assert_eq!(shares.len(), 3);
        let restored = combine_seed(&[shares[0], shares[2]], 2).unwrap();
        assert_eq!(restored.as_bytes(), seed.as_bytes());
        assert!(combine_seed(&shares[..1], 2).is_err());
    }
}
