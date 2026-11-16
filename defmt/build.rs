use std::{env, error::Error, fs, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    // Read the linker script
    let mut linker_script = fs::read_to_string("defmt.x.in")?;

    // Optionally exclude default panic handler
    if cfg!(feature = "avoid-default-panic") {
        linker_script = avoid_default_panic(linker_script);
    }

    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out.join("defmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());
    let target = env::var("TARGET")?;

    // `"atomic-cas": false` in `--print target-spec-json`
    // last updated: rust 1.48.0
    match &target[..] {
        "avr-gnu-base"
        | "msp430-none-elf"
        | "riscv32i-unknown-none-elf"
        | "riscv32imc-unknown-none-elf"
        | "thumbv4t-none-eabi"
        | "thumbv6m-none-eabi"
        | "xtensa-esp32s2-none-elf" => {
            println!("cargo:rustc-cfg=no_cas");
        }
        _ => {}
    }

    // Load string indices with a single `movw`, see `export::string_index!`.
    // Using symbols with quotes in `asm!(sym)` only works in Rust 1.91+.
    if has_movw(&target) && rustc_version().is_some_and(|v| v >= (1, 91)) {
        println!("cargo:rustc-cfg=defmt_movw");
    }

    // allow #[cfg(cfg_name)]
    for cfg_name in ["c_variadic", "defmt_movw", "no_cas"] {
        println!("cargo:rustc-check-cfg=cfg({cfg_name})");
    }

    Ok(())
}

/// Whether the target is Arm and supports the `movw` instruction (ARMv6T2+ or ARMv8-M Baseline).
///
/// Not supported: ARMv4T, ARMv5TE, ARMv6, ARMv6-M (`thumbv6m`).
/// last updated: rust 1.98.1
fn has_movw(target: &str) -> bool {
    ["armv7", "armv8", "armebv7", "thumbv7", "thumbv8"]
        .iter()
        .any(|prefix| target.starts_with(prefix))
}

fn rustc_version() -> Option<(u32, u32)> {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc).arg("--version").output().ok()?;
    // rustc 1.91.0-beta.1 (1bffa2300 2025-09-15)
    let version = String::from_utf8(output.stdout).ok()?;
    let mut parts = version.split_whitespace().nth(1)?.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    Some((major, minor))
}

fn avoid_default_panic(linker_script: String) -> String {
    linker_script.replacen("PROVIDE(_defmt_panic = __defmt_default_panic);", "", 1)
}
