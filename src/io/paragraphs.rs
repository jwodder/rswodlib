use std::io::{self, BufRead};
use std::iter::FusedIterator;

pub trait BufReadExt: BufRead {
    /// Returns an iterator over the paragraphs of this reader.
    ///
    /// Each paragraph is terminated by two or more consecutive newline
    /// sequences (LF or CR LF).  A single newline sequence at the start of a
    /// string is a paragraph by itself.  Trailing and embedded newline
    /// sequences in each paragraph are retained.
    ///
    /// If an error occurs while reading, the iterator will yield the error and
    /// then yield no more values.  If the error occurs after two or more
    /// consecutive newline sequences, the just-finished paragraph is yielded
    /// first; otherwise, if the error occurs in the middle of a paragraph, the
    /// paragraph in progress is discarded.
    fn paragraphs(self) -> Paragraphs<Self>
    where
        Self: Sized,
    {
        Paragraphs::new(self)
    }
}

impl<R: BufRead> BufReadExt for R {}

#[derive(Debug)]
pub struct Paragraphs<R>(State<R>);

#[derive(Debug)]
enum State<R> {
    Reading {
        inner: R,
        buffer: String,
        last_line_was_blank: bool,
    },
    Done(Option<io::Error>),
}

impl<R> Paragraphs<R> {
    fn new(inner: R) -> Paragraphs<R> {
        Paragraphs(State::Reading {
            inner,
            buffer: String::new(),
            last_line_was_blank: false,
        })
    }
}

impl<R: BufRead> Iterator for Paragraphs<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        match &mut self.0 {
            State::Reading {
                inner,
                buffer,
                last_line_was_blank,
            } => {
                loop {
                    let mut line = String::new();
                    match inner.read_line(&mut line) {
                        Ok(0) => {
                            // EOF
                            let r = (!buffer.is_empty()).then(|| Ok(std::mem::take(buffer)));
                            self.0 = State::Done(None);
                            return r;
                        }
                        Ok(_) => {
                            let is_blank = line == "\n" || line == "\r\n";
                            let r = (*last_line_was_blank && !is_blank)
                                .then(|| Ok(std::mem::take(buffer)));
                            buffer.push_str(&line);
                            *last_line_was_blank = is_blank;
                            if r.is_some() {
                                return r;
                            }
                        }
                        Err(e) => {
                            if *last_line_was_blank {
                                let r = Some(Ok(std::mem::take(buffer)));
                                self.0 = State::Done(Some(e));
                                return r;
                            } else {
                                self.0 = State::Done(None);
                                return Some(Err(e));
                            }
                        }
                    }
                }
            }
            State::Done(opt) => opt.take().map(Result::Err),
        }
    }
}

impl<R: BufRead> FusedIterator for Paragraphs<R> {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::Cursor;

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
            "This is a textual test.\r\rThis is the text that tests.\n\n\n",
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
    fn test_paragraphs(#[case] text: &str, #[case] paras: Vec<&str>) {
        let reader = Cursor::new(text);
        assert_eq!(
            reader.paragraphs().collect::<io::Result<Vec<_>>>().unwrap(),
            paras
        );
    }

    #[test]
    fn test_paragraphs_error_at_para_start() {
        let reader = Cursor::new(b"This is test text.\n\nThis is invalid UTF-8: f\xF6\xF6.\nThis is the line after the invalid UTF-8.\n\nThis is the paragraph after the invalid UTF-8.");
        let mut iter = reader.paragraphs();
        assert_eq!(iter.next().unwrap().unwrap(), "This is test text.\n\n");
        assert!(iter.next().unwrap().is_err());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_paragraphs_error_at_para_middle() {
        let reader = Cursor::new(b"This is test text.\n\nThis is the start of a new paragraph.\nThis is invalid UTF-8: f\xF6\xF6.\nThis is the line after the invalid UTF-8.\n\nThis is the paragraph after the invalid UTF-8.");
        let mut iter = reader.paragraphs();
        assert_eq!(iter.next().unwrap().unwrap(), "This is test text.\n\n");
        assert!(iter.next().unwrap().is_err());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
