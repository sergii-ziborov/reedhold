//! Integer helpers. Ads does not depend on the reputation crate.

/// Integer square root.
#[must_use]
pub fn isqrt(value: u32) -> u32 {
    if value <= 1 {
        return value;
    }
    let mut x = value;
    let mut y = x / 2;
    while y < x {
        x = y;
        y = x.saturating_add(value / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::isqrt;

    #[test]
    fn sqrt_of_one_hundred() {
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(2), 1);
    }
}
