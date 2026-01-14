use super::newlines::newlines;
use std::iter::FusedIterator;

/// Like [`str::lines`], except it consumes a `String` and yields `String`s,
/// and a lone CR is also treated as a newline sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringLines {
    content: String,
    offset: usize,
}

impl StringLines {
    pub fn new(content: String) -> StringLines {
        StringLines { content, offset: 0 }
    }
}

impl Iterator for StringLines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.offset == self.content.len() {
            return None;
        }
        let next_offset = newlines(&self.content[self.offset..])
            .next()
            .map_or(self.content.len(), |p| p.1 + self.offset);
        let mut end = next_offset;
        if end > 0 && self.content.as_bytes()[end - 1] == b'\n' {
            end -= 1;
        }
        if end > 0 && self.content.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        let line = &self.content[self.offset..end];
        self.offset = next_offset;
        Some(line.to_owned())
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
