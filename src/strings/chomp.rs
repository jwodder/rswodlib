/// Remove at most one trailing LF, CR LF, or CR from `s`
///
/// # Example
///
/// ```
/// # use rswodlib::strings::chomp::chomp;
/// assert_eq!(chomp("foo\r\n"), "foo");
/// assert_eq!(chomp("foo\n\n"), "foo\n");
/// ```
pub fn chomp(s: &str) -> &str {
    let s = s.strip_suffix('\n').unwrap_or(s);
    let s = s.strip_suffix('\r').unwrap_or(s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case("\n", "")]
    #[case("\r", "")]
    #[case("\r\n", "")]
    #[case("\n\n", "\n")]
    #[case("foo", "foo")]
    #[case("foo\n", "foo")]
    #[case("foo\r", "foo")]
    #[case("foo\r\n", "foo")]
    #[case("foo\n\r", "foo\n")]
    #[case("foo\n\n", "foo\n")]
    #[case("foo\nbar", "foo\nbar")]
    #[case("\nbar", "\nbar")]
    fn test_chomp(#[case] s1: &str, #[case] s2: &str) {
        assert_eq!(chomp(s1), s2);
    }
}
