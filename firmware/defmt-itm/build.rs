fn main() {
    println!("cargo:rustc-check-cfg=cfg(armv6m)");
}
