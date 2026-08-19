//! Integer milli-units. `1000` is a full factor. No floats.

/// Parts-per-thousand.
pub type Milli = u32;

/// Full factor.
pub const ONE: Milli = 1000;

/// Integer square root.
#[must_use]
pub fn isqrt(value: u32) -> u32 {
    if value <= 1 {
        return value;
    }
    let mut x = value;
    let mut y = x / 2 + 1;
    while y < x {
        x = y;
        y = x.saturating_add(value / x) / 2;
    }
    x
}

/// `a * b / 1000` saturating.
#[must_use]
pub fn mul(left: Milli, right: Milli) -> Milli {
    let product = u64::from(left).saturating_mul(u64::from(right)) / u64::from(ONE);
    u32::try_from(product).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{ONE, isqrt, mul};

    #[test]
    fn milli_mul_and_sqrt_are_integer() {
        assert_eq!(mul(ONE, ONE), ONE);
        assert_eq!(mul(500, 500), 250);
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(100), 10);
    }
}
