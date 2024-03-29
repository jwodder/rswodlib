use std::ops::{Div, Rem};

/// Compute both the integer quotient and the remainder of `dividend / divisor`
// cf. `div_rem()` and `Integer::div_rem()` from the `num` crate
pub fn divmod<T>(dividend: T, divisor: T) -> (T, T)
where
    T: Div<Output = T> + Rem<Output = T> + Copy,
{
    (dividend / divisor, dividend % divisor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divmod() {
        assert_eq!(divmod(5, 3), (1, 2));
        assert_eq!(divmod(5, -3), (-1, 2));
        assert_eq!(divmod(-5, 3), (-1, -2));
        assert_eq!(divmod(-5, -3), (1, -2));
    }
}
