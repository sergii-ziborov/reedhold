//! AES GF(256). Used only for Shamir shares of the master seed.

const IRRED: u8 = 0x1b;

pub(crate) fn mul(mut left: u8, mut right: u8) -> u8 {
    let mut product = 0_u8;
    for _ in 0..8 {
        if right & 1 != 0 {
            product ^= left;
        }
        let high = left & 0x80;
        left <<= 1;
        if high != 0 {
            left ^= IRRED;
        }
        right >>= 1;
    }
    product
}

pub(crate) fn inv(value: u8) -> Option<u8> {
    if value == 0 {
        return None;
    }
    let mut base = value;
    let mut acc = 1_u8;
    let mut exp = 254_u8;
    while exp > 0 {
        if exp & 1 != 0 {
            acc = mul(acc, base);
        }
        base = mul(base, base);
        exp >>= 1;
    }
    Some(acc)
}

pub(crate) fn eval(coeffs: &[u8], x: u8) -> u8 {
    let mut acc = 0_u8;
    for coeff in coeffs.iter().rev() {
        acc = mul(acc, x) ^ coeff;
    }
    acc
}

/// Lagrange interpolation of y(0) from distinct points.
pub(crate) fn interpolate_zero(points: &[(u8, u8)]) -> Option<u8> {
    let mut secret = 0_u8;
    for (i, (x_i, y_i)) in points.iter().enumerate() {
        let mut basis = 1_u8;
        for (j, (x_j, _)) in points.iter().enumerate() {
            if i == j {
                continue;
            }
            let denom = inv(x_j ^ x_i)?;
            basis = mul(basis, mul(*x_j, denom));
        }
        secret ^= mul(*y_i, basis);
    }
    Some(secret)
}

#[cfg(test)]
mod tests {
    use super::{eval, interpolate_zero};

    #[test]
    fn recovers_constant_term() {
        let coeffs = [0x42, 0x11, 0x07];
        let points = [
            (1, eval(&coeffs, 1)),
            (2, eval(&coeffs, 2)),
            (3, eval(&coeffs, 3)),
        ];
        assert_eq!(interpolate_zero(&points), Some(0x42));
        let linear = [0x42, 0x11];
        let pair = [(1, eval(&linear, 1)), (4, eval(&linear, 4))];
        assert_eq!(interpolate_zero(&pair), Some(0x42));
    }
}
