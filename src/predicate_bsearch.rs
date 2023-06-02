use std::ops::{Bound, RangeBounds};

/// Given a range in which there exists an `x` such that `!predicate(i)` for
/// all `i < x` in the range and `predicate(i)` for all `i >= x` in the range,
/// find & return `x` via a binary search.  Returns `None` if `x` could not be
/// found (meaning that some precondition was violated).
pub fn first_in_range<R, P>(range: R, mut predicate: P) -> Option<usize>
where
    R: RangeBounds<usize>,
    P: FnMut(usize) -> bool,
{
    let lower_bound = (range.start_bound().cloned(), Bound::Unbounded);
    let mut bounds = BinsearchBounds::from(range);
    while let Some(mid) = bounds.midpoint() {
        if predicate(mid) {
            if mid
                .checked_sub(1)
                .is_some_and(|prev| lower_bound.contains(&prev) && predicate(prev))
            {
                bounds = bounds.below(mid);
            } else {
                return Some(mid);
            }
        } else {
            bounds = bounds.above(mid);
        }
    }
    None
}

/// Given a range in which there exists an `x` such that `predicate(i)` for all
/// `i <= x` in the range and `!predicate(i)` for all `i > x` in the range,
/// find & return `x` via a binary search.  Returns `None` if `x` could not be
/// found (meaning that some precondition was violated).
pub fn last_in_range<R, P>(range: R, mut predicate: P) -> Option<usize>
where
    R: RangeBounds<usize>,
    P: FnMut(usize) -> bool,
{
    let upper_bound = (Bound::Unbounded, range.end_bound().cloned());
    let mut bounds = BinsearchBounds::from(range);
    while let Some(mid) = bounds.midpoint() {
        if predicate(mid) {
            if mid
                .checked_add(1)
                .is_some_and(|next| upper_bound.contains(&next) && predicate(next))
            {
                bounds = bounds.above(mid + 1);
            } else {
                return Some(mid);
            }
        } else {
            bounds = bounds.below(mid);
        }
    }
    None
}

struct BinsearchBounds {
    start: Bound<usize>,
    end: Bound<usize>,
}

impl BinsearchBounds {
    fn midpoint(&self) -> Option<usize> {
        let low = match self.start {
            Bound::Included(b) => b,
            Bound::Excluded(b) => b.checked_add(1)?,
            Bound::Unbounded => usize::MIN,
        };
        let high = match self.end {
            Bound::Included(b) => b,
            Bound::Excluded(b) => b.checked_sub(1)?,
            Bound::Unbounded => usize::MAX,
        };
        if low <= high {
            Some(low + (high - low) / 2)
        } else {
            None
        }
    }

    fn above(self, midpoint: usize) -> Self {
        BinsearchBounds {
            start: Bound::Excluded(midpoint),
            end: self.end,
        }
    }

    fn below(self, midpoint: usize) -> Self {
        BinsearchBounds {
            start: self.start,
            end: Bound::Excluded(midpoint),
        }
    }
}

impl<T: RangeBounds<usize>> From<T> for BinsearchBounds {
    fn from(bounds: T) -> BinsearchBounds {
        BinsearchBounds {
            start: bounds.start_bound().cloned(),
            end: bounds.end_bound().cloned(),
        }
    }
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
    fn test_first_in_range(#[case] low: usize, #[case] high: usize, #[case] answer: Option<usize>) {
        assert_eq!(first_in_range(low..high, |x| x > 7), answer);
    }

    #[rstest]
    #[case(1, 16, Some(6))]
    #[case(1, 6, Some(5))]
    #[case(6, 16, Some(6))]
    #[case(9, 16, None)]
    fn test_last_in_range(#[case] low: usize, #[case] high: usize, #[case] answer: Option<usize>) {
        assert_eq!(last_in_range(low..high, |x| x < 7), answer);
    }

    #[rstest]
    #[case(Bound::Included(0), Bound::Included(0), Some(0))]
    #[case(Bound::Included(0), Bound::Excluded(0), None)]
    #[case(Bound::Included(0), Bound::Excluded(1), Some(0))]
    #[case(Bound::Included(0), Bound::Included(1), Some(0))]
    #[case(Bound::Excluded(0), Bound::Excluded(1), None)]
    #[case(Bound::Excluded(0), Bound::Included(1), Some(1))]
    #[case(Bound::Included(0), Bound::Unbounded, Some(usize::MAX / 2))]
    #[case(Bound::Included(0), Bound::Excluded(usize::MAX), Some(usize::MAX / 2))]
    fn test_midpoint(
        #[case] low: Bound<usize>,
        #[case] high: Bound<usize>,
        #[case] mid: Option<usize>,
    ) {
        assert_eq!(BinsearchBounds::from((low, high)).midpoint(), mid);
    }
}
