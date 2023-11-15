use super::newlines::newlines;
use std::iter::FusedIterator;

/// Like [`str::lines`], except it consumes a `String` and yields `String`s,
/// and a lone CR is also treated as a newline sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringLines(String);

impl StringLines {
    pub fn new(content: String) -> StringLines {
        StringLines(content)
    }
}

impl Iterator for StringLines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }
        let end = newlines(&self.0)
            .next()
            .map(|p| p.1)
            .unwrap_or(self.0.len());
        let mut line: String = self.0.drain(0..end).collect();
        if line.ends_with('\n') {
            line.pop();
        }
        if line.ends_with('\r') {
            line.pop();
        }
        Some(line)
    }
}

impl FusedIterator for StringLines {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_lines() {
        let mut iter = StringLines::new("foo\r\nbar\n\nbaz\n".into());
        assert_eq!(iter.next().unwrap(), "foo");
        assert_eq!(iter.next().unwrap(), "bar");
        assert_eq!(iter.next().unwrap(), "");
        assert_eq!(iter.next().unwrap(), "baz");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn string_lines_no_final_newline() {
        let mut iter = StringLines::new("foo\nbar\n\r\nbaz".into());
        assert_eq!(iter.next().unwrap(), "foo");
        assert_eq!(iter.next().unwrap(), "bar");
        assert_eq!(iter.next().unwrap(), "");
        assert_eq!(iter.next().unwrap(), "baz");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
