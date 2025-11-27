fn main() {
    println!("cargo:rerun-if-env-changed=DEFMT_LOG");

    // Check rustc version
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .expect("failed to execute rustc");
    let version_str = String::from_utf8_lossy(&output.stdout);
    // rustc 1.91.0-beta.1 (1bffa2300 2025-09-15)
    let is_ge_1_91 = version_str
        .split_whitespace()
        .nth(1)
        .and_then(|v| {
            let mut parts = v.split('.');
            let major = parts.next()?.parse::<u32>().ok()?;
            let minor = parts.next()?.parse::<u32>().ok()?;
            Some((major, minor))
        })
        .map(|(major, minor)| major > 1 || (major == 1 && minor >= 91))
        .unwrap_or(false);
    if is_ge_1_91 {
        println!("cargo:rustc-cfg=rustc_ge_1_91");
    }
    println!("cargo:rustc-check-cfg=cfg(rustc_ge_1_91)");
}
