/// Divides a string in two before the first character that does not satisfy
/// the given predicate.  If the first part is nonempty, the parts are
/// returned.  Otherwise, `None` is returned.
///
/// Note that the first part is the maximal leading substring of `s` whose
/// characters all satisfy `predicate`.
///
/// # Example
///
/// ```
/// # use rswodlib::strings::span_some::span_some;
/// assert_eq!(span_some("123abc", |c| c.is_ascii_digit()), Some(("123", "abc")));
/// assert_eq!(span_some("abc123", |c| c.is_ascii_digit()), None);
/// ```
pub fn span_some<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> Option<(&str, &str)> {
    let boundary = s
        .char_indices()
        .find(move |&(_, ch)| !predicate(ch))
        .map_or(s.len(), |(i, _)| i);
    (boundary > 0).then(|| s.split_at(boundary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(
            span_some("123abc", |c| c.is_ascii_digit()),
            Some(("123", "abc"))
        );
    }

    #[test]
    fn all() {
        assert_eq!(
            span_some("123456", |c| c.is_ascii_digit()),
            Some(("123456", ""))
        );
    }

    #[test]
    fn none() {
        assert_eq!(span_some("abc123", |c| c.is_ascii_digit()), None);
    }
}
