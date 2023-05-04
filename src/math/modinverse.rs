use num_traits::int::PrimInt;
use num_traits::ops::euclid::Euclid;
use num_traits::sign::Signed;

/// `modinverse(a, n)` returns the [modular multiplicative inverse][1] of `a`
/// *modulo* `n`, i.e., the smallest positive integer `x` such that `(a *
/// x).rem_euclid(n) == 1`.  Returns `None` if `a` is not relatively prime to
/// `n` or if `n.abs() < 2`.
///
/// [1]: https://en.wikipedia.org/wiki/Modular_multiplicative_inverse
pub fn modinverse<T: PrimInt + Euclid + Signed>(a: T, n: T) -> Option<T> {
    let (mut upper, mut uc) = (n.abs(), T::zero());
    if upper < (T::one() + T::one()) {
        return None;
    }
    let (mut lower, mut lc) = (a.rem_euclid(&upper), T::one());
    while lower > T::one() {
        (upper, uc, lower, lc) = (
            lower,
            lc,
            upper.rem_euclid(&lower),
            uc - lc * (upper / lower),
        );
    }
    lower.is_one().then_some(lc.rem_euclid(&n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(3, 5, Some(2))]
    #[case(-2, 5, Some(2))]
    #[case(3, -5, Some(2))]
    #[case(-2, -5, Some(2))]
    #[case(8, 5, Some(2))]
    #[case(1, 5, Some(1))]
    #[case(2, 6, None)]
    #[case(0, 3, None)]
    #[case(5, 1, None)]
    #[case(5, 0, None)]
    fn test_modinverse(#[case] a: i32, #[case] n: i32, #[case] inv: Option<i32>) {
        assert_eq!(modinverse(a, n), inv);
    }
}
