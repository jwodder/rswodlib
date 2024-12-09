use std::iter::FusedIterator;

/// Returns an iterator over all pairs `(x, y)` where `x` is in `0..a` and `y`
/// is in `0..b`.
///
/// # Example
///
/// ```
/// # use rswodlib::cross_upto::cross_upto;
/// let mut iter = cross_upto(3, 2);
/// assert_eq!(iter.next(), Some((0, 0)));
/// assert_eq!(iter.next(), Some((0, 1)));
/// assert_eq!(iter.next(), Some((1, 0)));
/// assert_eq!(iter.next(), Some((1, 1)));
/// assert_eq!(iter.next(), Some((2, 0)));
/// assert_eq!(iter.next(), Some((2, 1)));
/// assert_eq!(iter.next(), None);
/// ```
pub fn cross_upto(a: usize, b: usize) -> CrossUpto {
    CrossUpto::new(a, b)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossUpto {
    max_a: usize,
    max_b: usize,
    a: usize,
    b: usize,
}

impl CrossUpto {
    fn new(max_a: usize, max_b: usize) -> CrossUpto {
        CrossUpto {
            max_a,
            max_b,
            a: 0,
            b: 0,
        }
    }
}

impl Iterator for CrossUpto {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.a >= self.max_a || self.b == self.max_b {
            return None;
        }
        let p = (self.a, self.b);
        self.b += 1;
        if self.b == self.max_b {
            self.b = 0;
            self.a += 1;
        }
        Some(p)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.a >= self.max_a || self.b == self.max_b {
            return (0, Some(0));
        }
        let sz = self.max_b * (self.max_a - self.a) - self.b;
        (sz, Some(sz))
    }
}

impl FusedIterator for CrossUpto {}

impl ExactSizeIterator for CrossUpto {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3x2() {
        let mut iter = cross_upto(3, 2);
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some((0, 0)));
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some((1, 0)));
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some((1, 1)));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some((2, 0)));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some((2, 1)));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_0x2() {
        let mut iter = cross_upto(0, 2);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_3x0() {
        let mut iter = cross_upto(3, 0);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_0x0() {
        let mut iter = cross_upto(0, 0);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
