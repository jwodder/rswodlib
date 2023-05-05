use std::borrow::Cow;

/// Convert all CR LF and CR sequences in a string to LF
pub fn normalize_newlines(s: &str) -> Cow<'_, str> {
    let mut buffer: Option<String> = None;
    let mut cr_prev = false;
    for (i, ch) in s.char_indices() {
        if ch == '\r' {
            if buffer.is_none() {
                let mut b = String::with_capacity(s.len());
                b.push_str(&s[..i]);
                buffer = Some(b);
            }
            buffer.as_mut().unwrap().push('\n');
            cr_prev = true;
        } else {
            if !(cr_prev && ch == '\n') {
                if let Some(b) = buffer.as_mut() {
                    b.push(ch);
                }
            }
            cr_prev = false;
        }
    }
    if let Some(b) = buffer {
        b.into()
    } else {
        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "", false)]
    #[case("\n", "\n", false)]
    #[case("\r\n", "\n", true)]
    #[case("\r", "\n", true)]
    #[case("foo", "foo", false)]
    #[case("foo\n", "foo\n", false)]
    #[case("foo\r\n", "foo\n", true)]
    #[case("foo\r\n\n", "foo\n\n", true)]
    #[case("foo\r", "foo\n", true)]
    #[case("foo\nbar\n", "foo\nbar\n", false)]
    #[case("foo\nbar\r\n", "foo\nbar\n", true)]
    #[case("foo\nbar\r", "foo\nbar\n", true)]
    #[case("foo\r\nbar\n", "foo\nbar\n", true)]
    #[case("foo\r\nbar\r\n", "foo\nbar\n", true)]
    #[case("foo\r\nbar\r", "foo\nbar\n", true)]
    #[case("foo\rbar\n", "foo\nbar\n", true)]
    #[case("foo\rbar\r\n", "foo\nbar\n", true)]
    #[case("foo\rbar\r", "foo\nbar\n", true)]
    fn test_normalize_newlines(#[case] s: &str, #[case] normed: &str, #[case] owned: bool) {
        let nn = normalize_newlines(s);
        assert_eq!(nn, normed);
        if owned {
            assert!(matches!(nn, Cow::Owned(_)));
        } else {
            assert!(matches!(nn, Cow::Borrowed(_)));
        }
    }
}
