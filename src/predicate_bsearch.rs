/// Given a range `low..high` in which there exists an `x` such that
/// `!predicate(i)` for all `i` in `low..x` and `predicate(i)` for all `i` in
/// `x..high`, find & return `x` via a binary search.  Returns `None` if `x`
/// could not be found (meaning that some precondition was violated).
pub fn find_first_property<P: FnMut(usize) -> bool>(
    mut predicate: P,
    low: usize,
    high: usize,
) -> Option<usize> {
    let mut lo = low;
    let mut hi = high;
    while lo < hi {
        let mid = hi - (hi - lo + 1) / 2;
        if predicate(mid) {
            if mid > low && predicate(mid - 1) {
                hi = mid
            } else {
                return Some(mid);
            }
        } else {
            lo = mid + 1;
        }
    }
    None
}

/// Given a range `low..high` in which there exists an `x` such that
/// `predicate(i)` for all `i` in `low..=x` and `!predicate(i)` for all `i` in
/// `(x+1)..high`, find & return `x` via a binary search.  Returns `None` if
/// `x` could not be found (meaning that some precondition was violated).
pub fn find_last_property<P: FnMut(usize) -> bool>(
    mut predicate: P,
    low: usize,
    high: usize,
) -> Option<usize> {
    let mut lo = low;
    let mut hi = high;
    while lo < hi {
        let mid = hi - (hi - lo + 1) / 2;
        if predicate(mid) {
            if mid + 1 < high && predicate(mid + 1) {
                lo = mid + 1;
            } else {
                return Some(mid);
            }
        } else {
            hi = mid;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1, 16, Some(8))]
    #[case(9, 16, Some(9))]
    #[case(1, 9, Some(8))]
    #[case(1, 8, None)]
    #[case(1, 5, None)]
    fn test_find_first_property(
        #[case] low: usize,
        #[case] high: usize,
        #[case] answer: Option<usize>,
    ) {
        assert_eq!(find_first_property(|x| x > 7, low, high), answer);
    }

    #[rstest]
    #[case(1, 16, Some(6))]
    #[case(1, 6, Some(5))]
    #[case(6, 16, Some(6))]
    #[case(9, 16, None)]
    fn test_find_last_property(
        #[case] low: usize,
        #[case] high: usize,
        #[case] answer: Option<usize>,
    ) {
        assert_eq!(find_last_property(|x| x < 7, low, high), answer);
    }
}
