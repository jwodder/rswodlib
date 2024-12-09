use std::fmt::{self, Write};
use std::time::Duration;

/// Returns a structure that displays the given [`Duration`] as a
/// floating-point number of seconds using no more precision than is necessary.
///
/// # Example
///
/// ```
/// # use rswodlib::show_duration::show_duration_as_seconds;
/// # use std::time::Duration;
/// let d1 = Duration::from_secs(42);
/// assert_eq!(show_duration_as_seconds(d1).to_string(), "42");
///
/// let d2 = Duration::from_nanos(123_000_000);
/// assert_eq!(show_duration_as_seconds(d2).to_string(), "0.123");
/// ```
pub fn show_duration_as_seconds(d: Duration) -> ShowDurationAsSeconds {
    ShowDurationAsSeconds(d)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ShowDurationAsSeconds(Duration);

impl fmt::Display for ShowDurationAsSeconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_secs())?;
        let nanos = self.0.subsec_nanos();
        if nanos > 0 {
            f.write_char('.')?;
            let mut frac = nanos;
            let mut divisor = 1_000_000_000 / 10;
            while frac > 0 && divisor > 0 {
                let d = frac / divisor;
                f.write_char(char::from_digit(d, 10).expect("should be valid decimal digit"))?;
                frac %= divisor;
                divisor /= 10;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0, "0")]
    #[case(0, 1, "0.000000001")]
    #[case(0, 100000000, "0.1")]
    #[case(0, 1000000, "0.001")]
    #[case(9, 999999999, "9.999999999")]
    #[case(10, 123456789, "10.123456789")]
    fn test(#[case] secs: u64, #[case] nanos: u32, #[case] s: &str) {
        let d = Duration::new(secs, nanos);
        assert_eq!(show_duration_as_seconds(d).to_string(), s);
    }
}
