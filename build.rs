use std::{
    collections::hash_map::DefaultHasher, env, error::Error, fs, path::PathBuf, process::Command,
    hash::Hasher,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let mut linker_script = fs::read_to_string("binfmt.x.in")?;
    let mut hasher = DefaultHasher::new();
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()?;
    hasher.write(&output.stdout);
    let hash = hasher.finish() as u32;
    linker_script = linker_script.replace("$BINFMT_VERSION", &hash.to_string());
    fs::write(out.join("binfmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());
    Ok(())
}
