use std::iter::{DoubleEndedIterator, FusedIterator};

/// Split a string into paragraphs, each one terminated by two or more newline
/// sequences (LF, CR LF, or CR).  A single newline sequence at the start of a
/// string is a paragraph by itself.  Trailing and embedded line endings in
/// each paragraph are retained.
pub fn split_paragraphs(s: &str) -> SplitParagraphs<'_> {
    SplitParagraphs(s)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SplitParagraphs<'a>(&'a str);

impl<'a> Iterator for SplitParagraphs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let mut newlines = 0;
        let mut cr_prev = false;
        let mut pos: Option<usize> = None;
        for (i, ch) in self.0.char_indices() {
            if ch != '\n' && ch != '\r' {
                if newlines > 1 {
                    break;
                } else {
                    newlines = 0;
                    cr_prev = false;
                    pos = None;
                }
            } else {
                if i == 0 {
                    // Pretend there was a newline before the start of the
                    // string so that a single newlines at the start will cause
                    // a new paragraph.
                    newlines += 1;
                }
                if ch == '\r' {
                    cr_prev = true;
                }
                if !(cr_prev && ch == '\n') {
                    newlines += 1;
                }
                if newlines > 1 {
                    pos = Some(i + 1);
                }
            }
        }
        let (s1, s2) = self.0.split_at(pos.unwrap_or(self.0.len()));
        self.0 = s2;
        Some(s1)
    }
}

impl<'a> FusedIterator for SplitParagraphs<'a> {}

impl<'a> DoubleEndedIterator for SplitParagraphs<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let mut newlines = 0;
        let mut lf_next = false;
        let mut pos: Option<usize> = None;
        let mut among_newlines = true;
        for (i, ch) in self.0.char_indices().rev() {
            if ch != '\n' && ch != '\r' {
                among_newlines = false;
                match (newlines, pos) {
                    (nls, Some(p)) if nls > 1 && p < self.0.len() => break,
                    _ => pos = None,
                }
            } else {
                if !std::mem::replace(&mut among_newlines, true) {
                    newlines = 0;
                    lf_next = false;
                    pos = None;
                }
                if pos.is_none() {
                    pos = Some(i + 1);
                }
                if ch == '\n' {
                    lf_next = true;
                }
                if !(lf_next && ch == '\r') {
                    newlines += 1;
                }
            }
        }
        if !(newlines == 1 && pos == Some(1) && self.0.len() != 1)
            && (newlines <= 1 || pos == Some(self.0.len()))
        {
            pos = Some(0);
        }
        let (s1, s2) = self.0.split_at(pos.unwrap_or(0));
        self.0 = s1;
        Some(s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", Vec::new())]
    #[case("\n", vec!["\n"])]
    #[case("\n\n", vec!["\n\n"])]
    #[case("\n\n\n", vec!["\n\n\n"])]
    #[case("This is test text.", vec!["This is test text."])]
    #[case("This is test text.\n", vec!["This is test text.\n"])]
    #[case("This is test text.\n\n", vec!["This is test text.\n\n"])]
    #[case("This is test text.\n\n\n", vec!["This is test text.\n\n\n"])]
    #[case(
        "This is test text.\nThis is a textual test.",
        vec!["This is test text.\nThis is a textual test."],
    )]
    #[case(
        "This is test text.\r\nThis is a textual test.",
        vec!["This is test text.\r\nThis is a textual test."],
    )]
    #[case(
        "This is test text.\n\nThis is a textual test.",
        vec!["This is test text.\n\n", "This is a textual test."],
    )]
    #[case(
        "This is test text.\n\n\nThis is a textual test.",
        vec!["This is test text.\n\n\n", "This is a textual test."],
    )]
    #[case("\nThis is test text.", vec!["\n", "This is test text."])]
    #[case("\n\nThis is test text.", vec!["\n\n", "This is test text."])]
    #[case("\n\n\nThis is test text.", vec!["\n\n\n", "This is test text."])]
    #[case(
        concat!(
            "This is test text.\r\n\r\nThis is a textual test.\r\r",
            "This is the text that tests.\n\n\n",
        ),
        vec![
            "This is test text.\r\n\r\n",
            "This is a textual test.\r\r",
            "This is the text that tests.\n\n\n",
        ],
    )]
    #[case(
        concat!(
            "This is test text.\r\nThis is a textual test.\r",
            "This is the text that tests.\n\n",
            "Boy, that Lorem Ipsum guy really had the right\nidea.\n\n",
        ),
        vec![
            concat!(
                "This is test text.\r\nThis is a textual test.\r",
                "This is the text that tests.\n\n",
            ),
            "Boy, that Lorem Ipsum guy really had the right\nidea.\n\n",
        ],
    )]
    #[case(
        "This is test text.\n\n \nThis is a textual test.\n",
        vec!["This is test text.\n\n", " \nThis is a textual test.\n"],
    )]
    #[case(
        "This is test text.\n\n \n\nThis is a textual test.\n",
        vec!["This is test text.\n\n", " \n\n", "This is a textual test.\n"],
    )]
    #[case(
        "This is test text.\n \n\nThis is a textual test.\n",
        vec!["This is test text.\n \n\n", "This is a textual test.\n"],
    )]
    fn test_split_paragraphs(#[case] text: &str, #[case] paras: Vec<&str>) {
        assert_eq!(split_paragraphs(text).collect::<Vec<_>>(), paras);
    }

    #[rstest]
    #[case("", Vec::new())]
    #[case("\n", vec!["\n"])]
    #[case("\n\n", vec!["\n\n"])]
    #[case("\n\n\n", vec!["\n\n\n"])]
    #[case("This is test text.", vec!["This is test text."])]
    #[case("This is test text.\n", vec!["This is test text.\n"])]
    #[case("This is test text.\n\n", vec!["This is test text.\n\n"])]
    #[case("This is test text.\n\n\n", vec!["This is test text.\n\n\n"])]
    #[case(
        "This is test text.\nThis is a textual test.",
        vec!["This is test text.\nThis is a textual test."],
    )]
    #[case(
        "This is test text.\r\nThis is a textual test.",
        vec!["This is test text.\r\nThis is a textual test."],
    )]
    #[case(
        "This is test text.\n\nThis is a textual test.",
        vec!["This is a textual test.", "This is test text.\n\n"],
    )]
    #[case(
        "This is test text.\n\n\nThis is a textual test.",
        vec!["This is a textual test.", "This is test text.\n\n\n"],
    )]
    #[case("\nThis is test text.", vec!["This is test text.", "\n"])]
    #[case("\n\nThis is test text.", vec!["This is test text.", "\n\n"])]
    #[case("\n\n\nThis is test text.", vec!["This is test text.", "\n\n\n"])]
    #[case(
        concat!(
            "This is test text.\r\n\r\nThis is a textual test.\r\r",
            "This is the text that tests.\n\n\n",
        ),
        vec![
            "This is the text that tests.\n\n\n",
            "This is a textual test.\r\r",
            "This is test text.\r\n\r\n",
        ],
    )]
    #[case(
        concat!(
            "This is test text.\r\nThis is a textual test.\r",
            "This is the text that tests.\n\n",
            "Boy, that Lorem Ipsum guy really had the right\nidea.\n\n",
        ),
        vec![
            "Boy, that Lorem Ipsum guy really had the right\nidea.\n\n",
            concat!(
                "This is test text.\r\nThis is a textual test.\r",
                "This is the text that tests.\n\n",
            ),
        ],
    )]
    #[case(
        "This is test text.\n\n \nThis is a textual test.\n",
        vec![" \nThis is a textual test.\n","This is test text.\n\n"],
    )]
    #[case(
        "This is test text.\n\n \n\nThis is a textual test.\n",
        vec!["This is a textual test.\n", " \n\n", "This is test text.\n\n"],
    )]
    #[case(
        "This is test text.\n \n\nThis is a textual test.\n",
        vec!["This is a textual test.\n", "This is test text.\n \n\n"],
    )]
    fn test_split_paragraphs_rev(#[case] text: &str, #[case] paras: Vec<&str>) {
        assert_eq!(split_paragraphs(text).rev().collect::<Vec<_>>(), paras);
    }
}
