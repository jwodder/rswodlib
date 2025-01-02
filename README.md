[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/rswodlib/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/rswodlib/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/rswodlib/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/rswodlib)
[![MIT License](https://img.shields.io/github/license/jwodder/rswodlib.svg)](https://opensource.org/licenses/MIT)

`rswodlib` is a personal collection of [Rust](https://www.rust-lang.org)
functions & functionalities that, though useful and often reused, aren't quite
worthy of published packages of their own.  It is not meant to be installed;
instead, if you see anything you like in it, you are encouraged to copy &
paste, subject to the MIT license.

The project is laid out as a [workspace][] in which the root package contains
only code with no (non-test) dependencies beyond `std` (and [`automod`][], for
convenience) while the other packages (all located in `crates/`) require one or
more third-party dependencies each.

[workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[`automod`]: https://crates.io/crates/automod
