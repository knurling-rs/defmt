use std::{env, error::Error, fs, path::PathBuf};

use git_version::git_version;

fn main() -> Result<(), Box<dyn Error>> {
    // `--match v*` matches defmt tags, e.g. `v0.2.3`, but ignores related crates, e.g. `defmt-decoder-v0.2.2`
    const GIT_DESCRIBE: &str = git_version!(args = ["--long", "--match", "v*"], cargo_prefix = "");

    let version = match semver::Version::parse(GIT_DESCRIBE) {
        // if parsing fails it is a git version
        Err(_) => extract_git_hash(GIT_DESCRIBE).to_string(),
        // if success it is semver
        Ok(semver) => match semver.major {
            // minor is breaking when major = 0
            0 => format!("{}.{}", semver.major, semver.minor),
            // ignore minor, patch, pre and build
            _ => semver.major.to_string(),
        },
    };

    // Load linker-script and insert version
    let mut linker_script = fs::read_to_string("defmt.x.in")?;
    linker_script = linker_script.replace("$DEFMT_VERSION", version.trim());

    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out.join("defmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());

    // `"atomic-cas": false` in `--print target-spec-json`
    // last updated: rust 1.48.0
    if matches!(
        &*env::var("TARGET")?,
        "avr-gnu-base"
            | "msp430-none-elf"
            | "riscv32i-unknown-none-elf"
            | "riscv32imc-unknown-none-elf"
            | "thumbv4t-none-eabi"
            | "thumbv6m-none-eabi"
    ) {
        println!("cargo:rustc-cfg=no_cas");
    }

    Ok(())
}

/// Extract git hash from a `git describe` statement
fn extract_git_hash(git_describe: &str) -> &str {
    git_describe.split("-").nth(2).unwrap()
}
