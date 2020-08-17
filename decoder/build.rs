use std::{env, error::Error, fs, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    let commit = String::from_utf8(output.stdout).unwrap();
    fs::write(
        out.join("version.rs"),
        format!(
            r#"
/// Supported `defmt` wire format
const DEFMT_VERSION: &str = "{}";
"#,
            commit.trim(),
        ),
    )?;
    Ok(())
}
