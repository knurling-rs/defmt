use std::{env, error::Error, fs, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let mut linker_script = fs::read_to_string("defmt.x.in")?;
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    let commit = String::from_utf8(output.stdout).unwrap();
    linker_script = linker_script.replace("$DEFMT_VERSION", commit.trim());
    fs::write(out.join("defmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());
    Ok(())
}
