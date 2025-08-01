#![cfg(nightly)]
#![feature(pattern)]
use std::str::pattern::{Pattern, Searcher};

/// If `pattern` occurs in `s`, returns a triple of the portion of `s` before
/// the pattern, the portion that matches the pattern, and the portion after
/// the pattern.
///
/// # Example
///
/// ```
/// # use rswodlib_partition::partition;
/// assert_eq!(partition("abc.123-xyz", ['-', '.']), Some(("abc", ".", "123-xyz")));
/// assert_eq!(partition("abc_123_xyz", ['-', '.']), None);
/// ```
pub fn partition<P: Pattern>(s: &str, pattern: P) -> Option<(&str, &str, &str)> {
    let mut searcher = pattern.into_searcher(s);
    let (start, end) = searcher.next_match()?;
    Some((&s[..start], &s[start..end], &s[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches() {
        assert_eq!(partition("abc-123-xyz", '-'), Some(("abc", "-", "123-xyz")));
    }

    #[test]
    fn does_not_match() {
        assert_eq!(partition("abc-123-xyz", ':'), None);
    }

    #[test]
    fn alternation_matches() {
        assert_eq!(
            partition("abc-123.xyz", ['.', '-']),
            Some(("abc", "-", "123.xyz"))
        );
    }
}
