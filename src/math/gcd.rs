use num_traits::int::PrimInt;
use num_traits::ops::euclid::Euclid;
use num_traits::sign::{Signed, Unsigned};

/// Compute the greatest common divisor of two unsigned integers.  If either
/// argument is zero, the other argument is returned.
pub fn gcd<T: PrimInt + Euclid + Unsigned>(mut a: T, mut b: T) -> T {
    if a.is_zero() {
        return b;
    } else if b.is_zero() {
        return a;
    }
    while !b.is_zero() {
        (a, b) = (b, a.rem_euclid(&b));
    }
    a
}

/// Compute the least common multiple of two unsigned integers.  If either
/// argument is zero, the result is zero.
pub fn lcm<T: PrimInt + Euclid + Unsigned>(a: T, b: T) -> T {
    let d = gcd(a, b);
    if d.is_zero() {
        d
    } else {
        (a * b).div_euclid(&d)
    }
}

/// Compute the greatest common divisor of two signed integers.  If either
/// argument is zero, the absolute value of the other argument is returned.
/// The result will always be nonnegative regardless of the signs of the
/// arguments.
pub fn gcd_signed<T: PrimInt + Euclid + Signed>(a: T, b: T) -> T {
    let mut a = a.abs();
    let mut b = b.abs();
    if a.is_zero() {
        return b;
    } else if b.is_zero() {
        return a;
    }
    while !b.is_zero() {
        (a, b) = (b, a.rem_euclid(&b));
    }
    a
}

/// Compute the least common multiple of two signed integers.  If either
/// argument is zero, the result is zero.  The result will always be
/// nonnegative regardless of the signs of the arguments.
pub fn lcm_signed<T: PrimInt + Euclid + Signed>(a: T, b: T) -> T {
    let d = gcd_signed(a, b);
    if d.is_zero() {
        d
    } else {
        (a * b).abs().div_euclid(&d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0, 0)]
    #[case(0, 4, 4)]
    #[case(4, 0, 4)]
    #[case(2, 4, 2)]
    #[case(2, 3, 1)]
    #[case(6, 4, 2)]
    fn test_gcd(#[case] a: u32, #[case] b: u32, #[case] c: u32) {
        assert_eq!(gcd(a, b), c);
    }

    #[rstest]
    #[case(0, 0, 0)]
    #[case(0, 4, 0)]
    #[case(4, 0, 0)]
    #[case(2, 4, 4)]
    #[case(2, 3, 6)]
    #[case(6, 4, 12)]
    fn test_lcm(#[case] a: u32, #[case] b: u32, #[case] c: u32) {
        assert_eq!(lcm(a, b), c);
    }

    #[rstest]
    #[case(0, 0, 0)]
    #[case(0, 4, 4)]
    #[case(4, 0, 4)]
    #[case(2, 4, 2)]
    #[case(2, 3, 1)]
    #[case(6, 4, 2)]
    #[case(-6, 4, 2)]
    #[case(6, -4, 2)]
    #[case(-6, -4, 2)]
    fn test_gcd_signed(#[case] a: i32, #[case] b: i32, #[case] c: i32) {
        assert_eq!(gcd_signed(a, b), c);
    }

    #[rstest]
    #[case(0, 0, 0)]
    #[case(0, 4, 0)]
    #[case(4, 0, 0)]
    #[case(2, 4, 4)]
    #[case(2, 3, 6)]
    #[case(6, 4, 12)]
    #[case(-6, 4, 12)]
    #[case(6, -4, 12)]
    #[case(-6, -4, 12)]
    fn test_lcm_signed(#[case] a: i32, #[case] b: i32, #[case] c: i32) {
        assert_eq!(lcm_signed(a, b), c);
    }
}
