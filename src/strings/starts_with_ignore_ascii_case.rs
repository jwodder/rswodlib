/// Returns `true` if `s` starts with `prefix`, ignoring ASCII case differences
pub fn starts_with_ignore_ascii_case(s: &str, prefix: &str) -> bool {
    s.get(..prefix.len())
        .is_some_and(|t| prefix.eq_ignore_ascii_case(t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Hello, World!", "hello", true)]
    #[case("Hello, World!", "HELLO", true)]
    #[case("hello, world!", "HELLO", true)]
    #[case("hello", "hello world", false)]
    #[case("Hellö, World!", "hello", false)]
    #[case("Hello, Wörld!", "hello", true)]
    #[case("", "hello", false)]
    #[case("Holla, World!", "hello", false)]
    #[case("Hello", "hello", true)]
    #[case("Hell", "hello", false)]
    fn test(#[case] s: &str, #[case] prefix: &str, #[case] r: bool) {
        assert_eq!(starts_with_ignore_ascii_case(s, prefix), r);
    }
}
