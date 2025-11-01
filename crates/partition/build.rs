use rustc_version::{Channel, version_meta};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    if version_meta().is_ok_and(|v| v.channel == Channel::Nightly) {
        println!("cargo::rustc-cfg=nightly");
    }
}
