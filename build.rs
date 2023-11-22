use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().is_ok_and(|v| v.channel == Channel::Nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
}
