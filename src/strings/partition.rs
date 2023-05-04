// Requires "pattern" feature on nightly
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
