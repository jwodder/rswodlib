/// Divides a string in two before the first character, counting from the
/// right/end, that does not satisfy the given predicate.  If the second part
/// is nonempty, the parts are returned.  Otherwise, `None` is returned.
///
/// Note that the second part is the maximal trailing substring of `s` whose
/// characters all satisfy `predicate`.
///
/// # Example
///
/// ```
/// # use rswodlib::strings::rspan_some::rspan_some;
/// assert_eq!(rspan_some("abc123", |c| c.is_ascii_digit()), Some(("abc", "123")));
/// assert_eq!(rspan_some("123abc", |c| c.is_ascii_digit()), None);
/// ```
pub fn rspan_some<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> Option<(&str, &str)> {
    s.char_indices()
        .rev()
        .take_while(move |&(_, ch)| predicate(ch))
        .last()
        .map(|(i, _)| s.split_at(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(
            rspan_some("abc123", |c| c.is_ascii_digit()),
            Some(("abc", "123"))
        );
    }

    #[test]
    fn all() {
        assert_eq!(
            rspan_some("123456", |c| c.is_ascii_digit()),
            Some(("", "123456"))
        );
    }

    #[test]
    fn none() {
        assert_eq!(rspan_some("123abc", |c| c.is_ascii_digit()), None);
    }
}
