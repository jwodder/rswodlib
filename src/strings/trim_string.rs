/// Mutate a `String` by removing all leading & trailing whitespace
pub fn trim_string(s: &mut String) {
    s.drain(..(s.len() - s.trim_start().len()));
    s.truncate(s.trim_end().len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case("foo", "foo")]
    #[case(" foo ", "foo")]
    #[case("foo ", "foo")]
    #[case(" foo", "foo")]
    #[case(" \t foo\r\n ", "foo")]
    #[case(" t foo\n. ", "t foo\n.")]
    fn test_trim_string(#[case] before: &str, #[case] after: &str) {
        let mut s = before.to_string();
        trim_string(&mut s);
        assert_eq!(s, after);
    }
}
