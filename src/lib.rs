#![feature(pattern)]
use std::str::pattern::Pattern;

// Requires "pattern" feature on nightly
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
    fn test_partition_matches() {
        assert_eq!(partition("abc-123-xyz", '-'), Some(("abc", "-", "123-xyz")));
    }

    #[test]
    fn test_partition_does_not_match() {
        assert_eq!(partition("abc-123-xyz", ':'), None);
    }

    #[test]
    fn test_partition_alternation_matches() {
        assert_eq!(
            partition("abc-123.xyz", ['.', '-']),
            Some(("abc", "-", "123.xyz"))
        );
    }
}
