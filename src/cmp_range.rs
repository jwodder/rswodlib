use std::cmp::Ordering;

/// # Panics
///
/// Panics if `lower > upper`
pub fn cmp_range<T: Ord + std::fmt::Debug>(value: T, lower: T, upper: T) -> RangeOrdering {
    assert!(
        lower <= upper,
        "cmp_range: expected lower <= upper; got lower={lower:?}, upper={upper:?}"
    );
    match (value.cmp(&lower), value.cmp(&upper)) {
        (Ordering::Less, _) => RangeOrdering::Less,
        (Ordering::Equal, Ordering::Less) => RangeOrdering::EqLower,
        (Ordering::Equal, Ordering::Equal) => RangeOrdering::EqBoth,
        (Ordering::Equal, Ordering::Greater) => unreachable!(),
        (Ordering::Greater, Ordering::Less) => RangeOrdering::Between,
        (Ordering::Greater, Ordering::Equal) => RangeOrdering::EqUpper,
        (Ordering::Greater, Ordering::Greater) => RangeOrdering::Greater,
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum RangeOrdering {
    Less,
    EqLower,
    Between,
    EqBoth,
    EqUpper,
    Greater,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_nontrivial_range() {
        use RangeOrdering::*;
        assert_eq!(cmp_range(1, 5, 10), Less);
        assert_eq!(cmp_range(4, 5, 10), Less);
        assert_eq!(cmp_range(5, 5, 10), EqLower);
        assert_eq!(cmp_range(6, 5, 10), Between);
        assert_eq!(cmp_range(10, 5, 10), EqUpper);
        assert_eq!(cmp_range(11, 5, 10), Greater);
        assert_eq!(cmp_range(15, 5, 10), Greater);
    }

    #[test]
    fn cmp_trivial_range() {
        use RangeOrdering::*;
        assert_eq!(cmp_range(1, 7, 7), Less);
        assert_eq!(cmp_range(6, 7, 7), Less);
        assert_eq!(cmp_range(7, 7, 7), EqBoth);
        assert_eq!(cmp_range(8, 7, 7), Greater);
        assert_eq!(cmp_range(10, 7, 7), Greater);
    }
}
