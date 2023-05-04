pub fn scan_some<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> Option<(&str, &str)> {
    let boundary = s
        .char_indices()
        .find(move |&(_, ch)| !predicate(ch))
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len());
    (boundary > 0).then(|| s.split_at(boundary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(
            scan_some("123abc", |c| c.is_ascii_digit()),
            Some(("123", "abc"))
        );
    }

    #[test]
    fn all() {
        assert_eq!(
            scan_some("123456", |c| c.is_ascii_digit()),
            Some(("123456", ""))
        );
    }

    #[test]
    fn none() {
        assert_eq!(scan_some("abc123", |c| c.is_ascii_digit()), None);
    }
}
