/// Divides a string in two before the first character that does not satisfy
/// the given predicate and returns the two parts.  (Note that the first part
/// is the maximal leading substring of `s` whose characters all satisfy
/// `predicate`.)
///
/// # Example
///
/// ```
/// # use rswodlib::strings::span::span;
/// assert_eq!(span("123abc", |c| c.is_ascii_digit()), ("123", "abc"));
/// assert_eq!(span("abc123", |c| c.is_ascii_digit()), ("", "abc123"));
/// ```
pub fn span<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> (&str, &str) {
    let boundary = s
        .char_indices()
        .find(move |&(_, ch)| !predicate(ch))
        .map_or(s.len(), |(i, _)| i);
    s.split_at(boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(span("123abc", |c| c.is_ascii_digit()), ("123", "abc"));
    }

    #[test]
    fn all() {
        assert_eq!(span("123456", |c| c.is_ascii_digit()), ("123456", ""));
    }

    #[test]
    fn none() {
        assert_eq!(span("abc123", |c| c.is_ascii_digit()), ("", "abc123"));
    }
}
