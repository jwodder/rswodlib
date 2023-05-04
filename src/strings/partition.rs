#![cfg(nightly)]
// Requires the "pattern" feature on nightly
// - `#![feature(pattern)]` must be enabled in the root module
// - rswodlib is configured to only make use of nightly features when actually
//   building on nightly.  To run a cargo command on nightly, insert `+nightly`
//   after `cargo`.
use std::str::pattern::Pattern;

/// If `pattern` occurs in `s`, returns a triple of the portion of `s` before
/// the pattern, the portion that matches the pattern, and the portion after
/// the pattern.
///
/// # Example
///
/// ```
/// # use rswodlib::strings::partition::partition;
/// assert_eq!(partition("abc.123-xyz", ['-', '.']), Some(("abc", ".", "123-xyz")));
/// assert_eq!(partition("abc_123_xyz", ['-', '.']), None);
/// ```
pub fn partition<'a, P: Pattern<'a>>(
    s: &'a str,
    pattern: P,
) -> Option<(&'a str, &'a str, &'a str)> {
    let (i, sep) = s.match_indices(pattern).next()?;
    Some((&s[..i], sep, &s[(i + sep.len())..]))
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
