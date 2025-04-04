use std::string::FromUtf8Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Utf8StrDecoder {
    partial: Vec<u8>,
}

impl Utf8StrDecoder {
    pub fn new() -> Self {
        Utf8StrDecoder {
            partial: Vec::with_capacity(3),
        }
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<String, FromUtf8Error> {
        let mut buf = Vec::with_capacity(self.partial.len().saturating_add(bytes.len()));
        buf.append(&mut self.partial);
        buf.extend(bytes.iter().copied());
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) if e.utf8_error().error_len().is_none() => {
                let good_len = e.utf8_error().valid_up_to();
                let mut buf = e.into_bytes();
                self.partial.extend(buf.drain(good_len..));
                let r = String::from_utf8(buf);
                debug_assert!(
                    r.is_ok(),
                    "valid_up_to should produce valid string but got {r:?}"
                );
                r
            }
            Err(e) => Err(e),
        }
    }

    pub fn finish(self) -> Result<(), FromUtf8Error> {
        if self.partial.is_empty() {
            Ok(())
        } else {
            let Err(e) = String::from_utf8(self.partial) else {
                unreachable!("nonempty partial UTF-8 sequence should not be valid");
            };
            Err(e)
        }
    }

    pub fn partial(&self) -> &[u8] {
        &self.partial
    }

    pub fn into_partial(self) -> Vec<u8> {
        self.partial
    }
}

impl Default for Utf8StrDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_ascii() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"Hello ".as_slice()).unwrap();
        assert_eq!(s, "Hello ");
        let s = decoder.decode(b"World".as_slice()).unwrap();
        assert_eq!(s, "World");
        assert!(decoder.finish().is_ok());
    }

    #[test]
    fn decode_good_utf8() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"H\xC3\xA9ll\xC3\xB6".as_slice()).unwrap();
        assert_eq!(s, "Héllö");
        let s = decoder.decode(b" W\xC3\xB8rl\xC3\xB0".as_slice()).unwrap();
        assert_eq!(s, " Wørlð");
        assert!(decoder.finish().is_ok());
    }

    #[test]
    fn decode_split_utf8() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"H\xC3".as_slice()).unwrap();
        assert_eq!(s, "H");
        let s = decoder.decode(b"\xA9ll\xC3\xB6".as_slice()).unwrap();
        assert_eq!(s, "éllö");
        let s = decoder.decode(b"!  How are you?".as_slice()).unwrap();
        assert_eq!(s, "!  How are you?");
        assert!(decoder.finish().is_ok());
    }

    #[test]
    fn invalid_utf8() {
        let mut decoder = Utf8StrDecoder::new();
        let e = decoder
            .decode(b"H\xC3\xC3ll\xC3\xB6".as_slice())
            .unwrap_err();
        assert_eq!(e.as_bytes(), b"H\xC3\xC3ll\xC3\xB6");
    }

    #[test]
    fn split_then_invalid() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"H\xC3".as_slice()).unwrap();
        assert_eq!(s, "H");
        let e = decoder.decode(b"\xA9ll\xC3\xC3".as_slice()).unwrap_err();
        assert_eq!(e.as_bytes(), b"\xC3\xA9ll\xC3\xC3");
    }

    #[test]
    fn invalid_split() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"H\xC3".as_slice()).unwrap();
        assert_eq!(s, "H");
        let e = decoder.decode(b"\xC3ll\xC3\xB6".as_slice()).unwrap_err();
        assert_eq!(e.as_bytes(), b"\xC3\xC3ll\xC3\xB6");
    }

    #[test]
    fn trailing_split() {
        let mut decoder = Utf8StrDecoder::new();
        let s = decoder.decode(b"H\xC3\xA9ll\xC3".as_slice()).unwrap();
        assert_eq!(s, "Héll");
        let e = decoder.finish().unwrap_err();
        assert_eq!(e.as_bytes(), b"\xC3");
    }
}
