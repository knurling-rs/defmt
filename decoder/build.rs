use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use semver::Version;

fn main() -> Result<(), Box<dyn Error>> {
    panic!("{:?}", env::current_dir());
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                assert!(!Path::new(".git").exists(), "you need to install the `git` command line tool to install the git version of `probe-run`");

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

    fs::write(
        out.join("version.rs"),
        format!(
            r#"
/// Supported `defmt` wire format
pub const DEFMT_VERSION: &str = "{}";
"#,
            version.trim(),
        ),
    )?;
    Ok(())
}
