/// Return the concatenation of the elements of `iter` with a linefeed inserted
/// after each one
pub fn unlines<I, S>(iter: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut s = String::new();
    for ln in iter {
        s.push_str(ln.as_ref());
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(unlines(std::iter::empty::<&str>()), "");
    }

    #[test]
    fn one() {
        assert_eq!(unlines(["foo"]), "foo\n");
    }

    #[test]
    fn many() {
        assert_eq!(
            unlines(["foo", "bar", "", "baz\n", "quux"]),
            "foo\nbar\n\nbaz\n\nquux\n"
        );
    }
}
