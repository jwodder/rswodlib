//! Functions for displaying a number and a noun with appropriate pluralization
use std::fmt;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Quantify<'a> {
    qty: usize,
    word: &'a str,
    ending: &'static str,
}

impl<'a> fmt::Display for Quantify<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}{}", self.qty, self.word, self.ending)
    }
}

/// Returns a structure that is [displayed][std::fmt::Display] as `"{qty}
/// {word}"`, with an S added to the end of `word` if `qty` is not 1.
///
/// If the plural of `word` is not formed by adding S, use [`quantify_irreg()`]
/// instead.
pub fn quantify(qty: usize, word: &str) -> Quantify<'_> {
    if qty == 1 {
        Quantify {
            qty,
            word,
            ending: "",
        }
    } else {
        Quantify {
            qty,
            word,
            ending: "s",
        }
    }
}

/// Returns a structure that is [displayed][std::fmt::Display] as `"{qty}
/// {singular}"` if `qty` is 1 or as `"{qty} {plural}"` otherwise.
pub fn quantify_irreg<'a>(qty: usize, singular: &'a str, plural: &'a str) -> Quantify<'a> {
    if qty == 1 {
        Quantify {
            qty,
            word: singular,
            ending: "",
        }
    } else {
        Quantify {
            qty,
            word: plural,
            ending: "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantify_one() {
        assert_eq!(quantify(1, "apple").to_string(), "1 apple");
    }

    #[test]
    fn quantify_zero() {
        assert_eq!(quantify(0, "apple").to_string(), "0 apples");
    }

    #[test]
    fn quantify_many() {
        assert_eq!(quantify(42, "apple").to_string(), "42 apples");
    }

    #[test]
    fn quantify_irreg_one() {
        assert_eq!(quantify_irreg(1, "mouse", "mice").to_string(), "1 mouse");
    }

    #[test]
    fn quantify_irreg_zero() {
        assert_eq!(quantify_irreg(0, "mouse", "mice").to_string(), "0 mice");
    }

    #[test]
    fn quantify_irreg_many() {
        assert_eq!(quantify_irreg(42, "mouse", "mice").to_string(), "42 mice");
    }
}
