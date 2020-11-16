use semver::Version;
use std::{env, error::Error, fs, path::Path, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let mut linker_script = fs::read_to_string("defmt.x.in")?;
    let hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                assert!(!Path::new(".git").exists(), "you need to install the `git` command line tool to use the git version of `defmt`");

                None
            }
        });
    let version = if let Ok(Some(hash)) = hash {
        hash
    } else {
        // no git info -> assume crates.io
        let semver = Version::parse(&std::env::var("CARGO_PKG_VERSION")?)?;
        if semver.major == 0 {
            // minor is breaking when major = 0
            format!("{}.{}", semver.major, semver.minor)
        } else {
            // ignore minor, patch, pre and build
            semver.major.to_string()
        }
    };
    linker_script = linker_script.replace("$DEFMT_VERSION", version.trim());
    fs::write(out.join("defmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());
    Ok(())
}
