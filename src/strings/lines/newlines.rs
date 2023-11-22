use std::iter::FusedIterator;

/// Given a string, returns an iterator that yields the start & end indices of
/// every newline sequence (LF, CR LF, or CR) in the string.
pub fn newlines(s: &str) -> Newlines<'_> {
    Newlines::new(s)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Newlines<'a> {
    s: &'a str,
    taken: usize,
}

impl<'a> Newlines<'a> {
    fn new(s: &'a str) -> Newlines<'a> {
        Newlines { s, taken: 0 }
    }
}

impl Iterator for Newlines<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let start = self.s.find(['\n', '\r'])?;
        let end = {
            if self.s.get(start..(start + 2)) == Some("\r\n") {
                start + 2
            } else {
                start + 1
            }
        };
        let r = (start + self.taken, end + self.taken);
        self.s = &self.s[end..];
        self.taken += end;
        Some(r)
    }
}

impl DoubleEndedIterator for Newlines<'_> {
    fn next_back(&mut self) -> Option<(usize, usize)> {
        let penult = self.s.rfind(['\n', '\r'])?;
        let end = penult + 1;
        let start = match penult.checked_sub(1) {
            Some(i) if self.s.get(i..end) == Some("\r\n") => i,
            _ => penult,
        };
        self.s = &self.s[..start];
        Some((start + self.taken, end + self.taken))
    }
}

impl FusedIterator for Newlines<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_empty() {
        let mut iter = newlines("");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_no_newline() {
        let mut iter = newlines("foobar");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[rstest]
    #[case("\n", (0, 1))]
    #[case("\r", (0, 1))]
    #[case("\r\n", (0, 2))]
    #[case("foo\n", (3, 4))]
    #[case("foo\r", (3, 4))]
    #[case("foo\r\n", (3, 5))]
    #[case("\nfoo", (0, 1))]
    #[case("\rfoo", (0, 1))]
    #[case("\r\nfoo", (0, 2))]
    #[case("foo\nbar", (3, 4))]
    #[case("foo\rbar", (3, 4))]
    #[case("foo\r\nbar", (3, 5))]
    #[case("foo“\n”bar", (6, 7))]
    #[case("foo“\r”bar", (6, 7))]
    #[case("foo“\r\n”bar", (6, 8))]
    fn test_one_newline(#[case] s: &str, #[case] value: (usize, usize)) {
        let mut iter = newlines(s);
        assert_eq!(iter.next(), Some(value));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next_back(), None);
        let mut riter = newlines(s);
        assert_eq!(riter.next_back(), Some(value));
        assert_eq!(riter.next_back(), None);
        assert_eq!(riter.next_back(), None);
        assert_eq!(riter.next(), None);
        assert_eq!(riter.next(), None);
    }

    #[rstest]
    #[case("\n\r", (0, 1), (1, 2))]
    #[case("foo\n\rbar", (3, 4), (4, 5))]
    #[case("foo\n\nbar", (3, 4), (4, 5))]
    #[case("foo\r\rbar", (3, 4), (4, 5))]
    #[case("foo\nbar\n", (3, 4), (7, 8))]
    #[case("foo\rbar\r", (3, 4), (7, 8))]
    #[case("foo\r\nbar\r\n", (3, 5), (8, 10))]
    fn test_two_newlines(
        #[case] s: &str,
        #[case] nel1: (usize, usize),
        #[case] nel2: (usize, usize),
    ) {
        let mut iter = newlines(s);
        assert_eq!(iter.next(), Some(nel1));
        assert_eq!(iter.next(), Some(nel2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next_back(), None);
        let mut riter = newlines(s);
        assert_eq!(riter.next_back(), Some(nel2));
        assert_eq!(riter.next_back(), Some(nel1));
        assert_eq!(riter.next_back(), None);
        assert_eq!(riter.next_back(), None);
        assert_eq!(riter.next(), None);
        assert_eq!(riter.next(), None);
        let mut diter = newlines(s);
        assert_eq!(diter.next(), Some(nel1));
        assert_eq!(diter.next_back(), Some(nel2));
        assert_eq!(diter.next(), None);
        assert_eq!(diter.next(), None);
        assert_eq!(diter.next_back(), None);
        assert_eq!(diter.next_back(), None);
    }
}
