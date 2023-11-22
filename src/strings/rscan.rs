/// Divides a string in two before the first character, counting from the
/// right/end, that does not satisfy the given predicate and returns the two
/// parts.  (Note that the second part is the maximal trailing substring of `s`
/// whose characters all satisfy `predicate`.)
///
/// # Example
///
/// ```
/// # use rswodlib::strings::rscan::rscan;
/// assert_eq!(rscan("abc123", |c| c.is_ascii_digit()), ("abc", "123"));
/// assert_eq!(rscan("123abc", |c| c.is_ascii_digit()), ("123abc", ""));
/// ```
pub fn rscan<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> (&str, &str) {
    let boundary = s
        .char_indices()
        .rev()
        .take_while(move |&(_, ch)| predicate(ch))
        .last()
        .map_or(s.len(), |(i, _)| i);
    s.split_at(boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(rscan("abc123", |c| c.is_ascii_digit()), ("abc", "123"));
    }

    #[test]
    fn all() {
        assert_eq!(rscan("123456", |c| c.is_ascii_digit()), ("", "123456"));
    }

    #[test]
    fn none() {
        assert_eq!(rscan("123abc", |c| c.is_ascii_digit()), ("123abc", ""));
    }
}
