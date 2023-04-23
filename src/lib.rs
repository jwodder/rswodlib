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

pub fn scan<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> (&str, &str) {
    let boundary = s
        .char_indices()
        .take_while(move |&(_, ch)| predicate(ch))
        .last()
        .map(|(i, ch)| i + ch.len_utf8())
        .unwrap_or_default();
    s.split_at(boundary)
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

    #[test]
    fn test_scan_half() {
        assert_eq!(scan("123abc", |c| c.is_ascii_digit()), ("123", "abc"));
    }

    #[test]
    fn test_scan_all() {
        assert_eq!(scan("123456", |c| c.is_ascii_digit()), ("123456", ""));
    }

    #[test]
    fn test_scan_none() {
        assert_eq!(scan("abc123", |c| c.is_ascii_digit()), ("", "abc123"));
    }
}
