use std::{
    collections::hash_map::DefaultHasher, env, error::Error, fs, hash::Hasher, path::PathBuf,
    process::Command,
};

fn main() -> Result<(), Box<dyn Error>> {
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let mut hasher = DefaultHasher::new();
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    hasher.write(&output.stdout);
    let hash = hasher.finish() as u32;
    fs::write(
        out.join("version.rs"),
        format!("\
/// Supported `binfmt` wire format
const BINFMT_VERSION: usize = {};", hash),
    )?;
    Ok(())
}
