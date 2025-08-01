This crate defines a `CommandExt` extension trait for `std::process::Command`
that adds a `combined_output()` method for running a command and capturing its
combined stdout and stderr.

This crate does not use [`std::io::pipe()`], and thus it works with versions of
Rust older than 1.87.
