use std::io::{ErrorKind, Read};

/// Read bytes into `buf`, stopping only when `buf` is filled, EOF is reached,
/// or an error not of the kind [`ErrorKind::Interrupted`] is encountered.  On
/// success, returns the number of bytes read into `buf`.
///
/// Unlike [`Read::read_exact()`], EOF is not regarded as an error.
pub fn read_up_to<R: Read>(mut reader: R, mut buf: &mut [u8]) -> std::io::Result<usize> {
    let mut bytes = 0;
    while !buf.is_empty() {
        match reader.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                buf = &mut buf[n..];
                bytes += n;
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_some() {
        let mut reader = Cursor::new("Hello, world!");
        let mut buf = vec![0u8; 6];
        let n = read_up_to(&mut reader, &mut buf).unwrap();
        assert_eq!(n, 6);
        assert_eq!(buf, b"Hello,");
    }

    #[test]
    fn read_to_eof() {
        let mut reader = Cursor::new("Hello");
        let mut buf = vec![0u8; 6];
        let n = read_up_to(&mut reader, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf[..n], b"Hello");
    }
}
