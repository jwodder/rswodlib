use super::newlines::newlines;
use std::iter::FusedIterator;

/// Like [`str::lines`], except the terminating newlines are retained, and a
/// lone CR is also treated as a newline sequence.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use rswodlib::strings::lines::lines_keepends::lines_keepends;
/// let text = "foo\r\nbar\n\rbaz\n";
/// let mut lines = lines_keepends(text);
///
/// assert_eq!(Some("foo\r\n"), lines.next());
/// assert_eq!(Some("bar\n"), lines.next());
/// assert_eq!(Some("\r"), lines.next());
/// assert_eq!(Some("baz\n"), lines.next());
/// assert_eq!(None, lines.next());
/// ```
///
/// The final line ending isn't required:
///
/// ```
/// # use rswodlib::strings::lines::lines_keepends::lines_keepends;
/// let text = "foo\nbar\n\r\nbaz";
/// let mut lines = lines_keepends(text);
///
/// assert_eq!(Some("foo\n"), lines.next());
/// assert_eq!(Some("bar\n"), lines.next());
/// assert_eq!(Some("\r\n"), lines.next());
/// assert_eq!(Some("baz"), lines.next());
/// assert_eq!(None, lines.next());
/// ```
pub fn lines_keepends(s: &str) -> LinesKeepends<'_> {
    LinesKeepends(s)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinesKeepends<'a>(&'a str);

impl<'a> Iterator for LinesKeepends<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let pos = match newlines(self.0).next() {
            Some((_, end)) => end,
            None => self.0.len(),
        };
        let (s1, s2) = self.0.split_at(pos);
        self.0 = s2;
        Some(s1)
    }
}

impl<'a> FusedIterator for LinesKeepends<'a> {}

impl<'a> DoubleEndedIterator for LinesKeepends<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let length = self.0.len();
        let pos = newlines(self.0)
            .rev()
            .map(|p| p.1)
            .find(|&end| end != length)
            .unwrap_or_default();
        let (s1, s2) = self.0.split_at(pos);
        self.0 = s1;
        Some(s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lines_keepends() {
        let mut iter = lines_keepends("foo\r\nbar\n\rbaz\n");
        assert_eq!(iter.next(), Some("foo\r\n"));
        assert_eq!(iter.next(), Some("bar\n"));
        assert_eq!(iter.next(), Some("\r"));
        assert_eq!(iter.next(), Some("baz\n"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_no_terminator() {
        let mut iter = lines_keepends("foo\nbar\n\r\nbaz");
        assert_eq!(iter.next(), Some("foo\n"));
        assert_eq!(iter.next(), Some("bar\n"));
        assert_eq!(iter.next(), Some("\r\n"));
        assert_eq!(iter.next(), Some("baz"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_no_newline() {
        let mut iter = lines_keepends("foobar");
        assert_eq!(iter.next(), Some("foobar"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_empty() {
        let mut iter = lines_keepends("");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_rev() {
        let mut iter = lines_keepends("foo\r\nbar\n\rbaz\n").rev();
        assert_eq!(iter.next(), Some("baz\n"));
        assert_eq!(iter.next(), Some("\r"));
        assert_eq!(iter.next(), Some("bar\n"));
        assert_eq!(iter.next(), Some("foo\r\n"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_no_terminator_rev() {
        let mut iter = lines_keepends("foo\nbar\n\r\nbaz").rev();
        assert_eq!(iter.next(), Some("baz"));
        assert_eq!(iter.next(), Some("\r\n"));
        assert_eq!(iter.next(), Some("bar\n"));
        assert_eq!(iter.next(), Some("foo\n"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_no_newline_rev() {
        let mut iter = lines_keepends("foobar").rev();
        assert_eq!(iter.next(), Some("foobar"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines_keepends_empty_rev() {
        let mut iter = lines_keepends("").rev();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
