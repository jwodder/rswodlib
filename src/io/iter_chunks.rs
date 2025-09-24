use super::read_up_to::read_up_to;
use std::io;

/// Returns an iterator that reads & yields bytes from `reader` in chunks of
/// length `chunk_size` (except for the final chunk, which may be shorter).
pub fn iter_chunks<R>(reader: R, chunk_size: usize) -> IterChunks<R> {
    IterChunks { reader, chunk_size }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IterChunks<R> {
    reader: R,
    chunk_size: usize,
}

impl<R: io::Read> Iterator for IterChunks<R> {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = vec![0u8; self.chunk_size];
        match read_up_to(&mut self.reader, &mut buf) {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn exact() {
        let reader = Cursor::new("Hello, world");
        let mut iter = iter_chunks(reader, 4);
        assert_eq!(iter.next().unwrap().unwrap(), b"Hell");
        assert_eq!(iter.next().unwrap().unwrap(), b"o, w");
        assert_eq!(iter.next().unwrap().unwrap(), b"orld");
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn uneven() {
        let reader = Cursor::new("Hello, world!");
        let mut iter = iter_chunks(reader, 5);
        assert_eq!(iter.next().unwrap().unwrap(), b"Hello");
        assert_eq!(iter.next().unwrap().unwrap(), b", wor");
        assert_eq!(iter.next().unwrap().unwrap(), b"ld!");
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
