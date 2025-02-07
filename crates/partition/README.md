This crate defines a `partition()` function modelled on Python's
[`str.partition()`][1] for splitting a `&str` on the first occurrence of a
pattern and returning the part before the matching substring, the matching
substring, and the part after the matching substring.

Because this crate uses the unstable [`Pattern`][2] trait, it can only be built
on nightly Rust.  If built on stable Rust, the `build.rs` script will disable
all functionality.

[1]: https://docs.python.org/3/library/stdtypes.html#str.partition
[2]: https://doc.rust-lang.org/std/str/pattern/trait.Pattern.html
