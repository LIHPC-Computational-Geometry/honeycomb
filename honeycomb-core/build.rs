#[rustversion::nightly]
fn set_rustc_channel_cfg() -> &'static str {
    "nightly"
}

#[rustversion::beta]
fn set_rustc_channel_cfg() -> &'static str {
    "beta"
}

#[rustversion::stable]
fn set_rustc_channel_cfg() -> &'static str {
    "stable"
}

fn main() {
    println!("cargo:rustc-cfg={}", set_rustc_channel_cfg());
}
