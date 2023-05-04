#![cfg(nightly)]
// Requires the "pattern" feature on nightly
// - `#![feature(pattern)]` must be enabled in the root module
// - rswodlib is configured to only make use of nightly features when actually
//   building on nightly.  To run a cargo command on nightly, insert `+nightly`
//   after `cargo`.
use std::str::pattern::Pattern;

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
