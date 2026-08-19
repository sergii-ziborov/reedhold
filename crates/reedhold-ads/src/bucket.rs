//! Coarse reputation buckets. Inventory is sold by bucket, never by user id.

/// Map 0..=10000 strength onto B0..=B5.
#[must_use]
pub fn bucket(strength: u32) -> u8 {
    match strength {
        0..=999 => 0,
        1000..=2499 => 1,
        2500..=4999 => 2,
        5000..=7499 => 3,
        7500..=8999 => 4,
        _ => 5,
    }
}

/// Minimum bid for this bucket, scaled by creative risk (milli, 1000 = 1.0).
#[must_use]
pub fn floor(bucket: u8, risk_milli: u32) -> u64 {
    let base = 10_u64.saturating_mul(u64::from(bucket.saturating_add(1)));
    let risk = u64::from(risk_milli.max(1000));
    base.saturating_mul(risk) / 1000
}

/// Strength needed before an identity may distribute.
pub const DISTRIBUTOR_MIN_BUCKET: u8 = 2;

#[cfg(test)]
mod tests {
    use super::{bucket, floor};

    #[test]
    fn stronger_bucket_has_a_higher_floor() {
        assert!(floor(bucket(9500), 1000) > floor(bucket(100), 1000));
        assert!(floor(5, 2000) > floor(5, 1000));
    }
}
