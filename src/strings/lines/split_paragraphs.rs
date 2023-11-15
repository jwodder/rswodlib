use super::newlines::newlines;
use std::iter::FusedIterator;

/// Split a string into paragraphs, each one terminated by two or more
/// consecutive newline sequences (LF, CR LF, or CR).  A single newline
/// sequence at the start of a string is a paragraph by itself.  Trailing and
/// embedded newline sequences in each paragraph are retained.
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
        let mut nlqty = 0;
        let mut length = None;
        let mut at0 = false;
        for (start, end) in newlines(self.0) {
            if length == Some(start) {
                length = Some(end);
                nlqty += 1;
            } else if nlqty > 1 || at0 {
                let (s1, s2) = self.0.split_at(length.unwrap());
                self.0 = s2;
                return Some(s1);
            } else {
                length = Some(end);
                nlqty = 1;
                at0 = start == 0;
            }
        }
        if nlqty > 1 || at0 {
            let (s1, s2) = self.0.split_at(length.unwrap());
            self.0 = s2;
            Some(s1)
        } else {
            Some(std::mem::take(&mut self.0))
        }
    }
}

impl<'a> FusedIterator for SplitParagraphs<'a> {}

impl<'a> DoubleEndedIterator for SplitParagraphs<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let mut nlqty = 0;
        let mut para_end = None;
        let mut para_sep_start = None;
        for (start, end) in newlines(self.0).rev() {
            if para_sep_start == Some(end) {
                para_sep_start = Some(start);
                nlqty += 1;
            } else if nlqty > 1 && para_end != Some(self.0.len()) {
                let (s1, s2) = self.0.split_at(para_end.unwrap());
                self.0 = s1;
                return Some(s2);
            } else {
                para_end = Some(end);
                para_sep_start = Some(start);
                nlqty = 1;
            }
        }
        if para_end != Some(self.0.len()) && (nlqty > 1 || para_sep_start == Some(0)) {
            let (s1, s2) = self.0.split_at(para_end.unwrap());
            self.0 = s1;
            Some(s2)
        } else {
            Some(std::mem::take(&mut self.0))
        }
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
