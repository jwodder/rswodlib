use super::rscan_some::rscan_some;
use super::scan_some::scan_some;

/// Mutate a `String` by removing all leading & trailing whitespace
pub fn trim_string(s: &mut String) {
    if let Some((leading_ws, _)) = scan_some(s, char::is_whitespace) {
        s.drain(..leading_ws.len());
    }
    if let Some((core, _)) = rscan_some(s, char::is_whitespace) {
        s.truncate(core.len());
    }
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
