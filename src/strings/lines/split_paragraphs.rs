use super::newlines::newlines;
use std::iter::FusedIterator;

/// Split a string into paragraphs, each one terminated by two or more
/// consecutive newline sequences (LF, CR LF, or CR).  A single newline
/// sequence at the start of a string is a paragraph by itself.  Trailing and
/// embedded newline sequences in each paragraph are retained.
pub fn split_paragraphs(s: &str) -> SplitParagraphs<'_> {
    SplitParagraphs(s)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SplitParagraphs<'a>(&'a str);

impl<'a> Iterator for SplitParagraphs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let mut tracker = Tracker::new();
        let pos = newlines(self.0)
            .find_map(|span| tracker.handle(span))
            .or_else(|| tracker.end())
            .unwrap_or(self.0.len());
        let (s1, s2) = self.0.split_at(pos);
        self.0 = s2;
        Some(s1)
    }
}

impl FusedIterator for SplitParagraphs<'_> {}

impl<'a> DoubleEndedIterator for SplitParagraphs<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if self.0.is_empty() {
            return None;
        }
        let mut tracker = RevTracker::new(self.0);
        let pos = newlines(self.0)
            .rev()
            .find_map(|span| tracker.handle(span))
            .or_else(|| tracker.end())
            .unwrap_or_default();
        let (s1, s2) = self.0.split_at(pos);
        self.0 = s1;
        Some(s2)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Tracker(Option<TrackerState>);

impl Tracker {
    fn new() -> Tracker {
        Tracker(None)
    }

    fn handle(&mut self, (start, end): (usize, usize)) -> Option<usize> {
        let ending = self.end();
        match (&mut self.0, ending) {
            (Some(st), _) if st.para_sep_end == start => {
                st.para_sep_end = end;
                st.newlines += 1;
                None
            }
            (_, r @ Some(_)) => r,
            (st, _) => {
                *st = Some(TrackerState {
                    para_sep_start: start,
                    para_sep_end: end,
                    newlines: 1,
                });
                None
            }
        }
    }

    fn end(&self) -> Option<usize> {
        self.0
            .filter(TrackerState::is_para_ending)
            .map(|st| st.para_sep_end)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct RevTracker {
    state: Option<TrackerState>,
    strlen: usize,
}

impl RevTracker {
    fn new(s: &str) -> RevTracker {
        RevTracker {
            state: None,
            strlen: s.len(),
        }
    }

    fn handle(&mut self, (start, end): (usize, usize)) -> Option<usize> {
        let ending = self.end();
        match (&mut self.state, ending) {
            (Some(st), _) if st.para_sep_start == end => {
                st.para_sep_start = start;
                st.newlines += 1;
                None
            }
            (_, r @ Some(_)) => r,
            (st, _) => {
                *st = Some(TrackerState {
                    para_sep_start: start,
                    para_sep_end: end,
                    newlines: 1,
                });
                None
            }
        }
    }

    fn end(&self) -> Option<usize> {
        self.state
            .filter(|st| st.is_para_ending() && st.para_sep_end != self.strlen)
            .map(|st| st.para_sep_end)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct TrackerState {
    para_sep_start: usize,
    para_sep_end: usize,
    newlines: usize,
}

impl TrackerState {
    fn is_para_ending(&self) -> bool {
        self.newlines > 1 || self.para_sep_start == 0
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
