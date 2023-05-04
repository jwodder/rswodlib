pub fn scan<P: FnMut(char) -> bool>(s: &str, mut predicate: P) -> (&str, &str) {
    let boundary = s
        .char_indices()
        .find(move |&(_, ch)| !predicate(ch))
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len());
    s.split_at(boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half() {
        assert_eq!(scan("123abc", |c| c.is_ascii_digit()), ("123", "abc"));
    }

    #[test]
    fn all() {
        assert_eq!(scan("123456", |c| c.is_ascii_digit()), ("123456", ""));
    }

    #[test]
    fn none() {
        assert_eq!(scan("abc123", |c| c.is_ascii_digit()), ("", "abc123"));
    }
}
