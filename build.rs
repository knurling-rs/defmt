use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    fs::copy("binfmt.x", out.join("binfmt.x"))?;
    println!("cargo:rustc-link-search={}", out.display());
    Ok(())
}
