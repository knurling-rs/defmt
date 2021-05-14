use std::{collections::HashSet, process::Command};

use crate::run_capturing_stdout;

/// Make sure a fixed set of compilation targets is installed
///
/// Returns the `added_targets`.
pub fn install() -> anyhow::Result<Vec<String>> {
    let required_targets = [
        "thumbv6m-none-eabi",
        "thumbv7m-none-eabi",
        "thumbv7em-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ]
    .iter()
    .map(|item| item.to_string())
    .collect::<HashSet<_>>();

    // the `added_targets` will potentially get uninstalled later
    let added_targets = required_targets.difference(&get_installed()?).cloned().collect();

    // install _all_ required targets; previously installed targets will get updated
    println!("⏳ installing targets");
    let status = Command::new("rustup")
        .args(&["target", "add"])
        .args(&required_targets)
        .status()?;
    if !status.success() {
        // since installing targets is the first thing we do, hard panic is OK enough (user would notice at this point)
        panic!("Error installing targets (see output above)");
    }

    Ok(added_targets)
}

/// Get all currently installed compilation targets
fn get_installed() -> anyhow::Result<HashSet<String>> {
    let stdout = run_capturing_stdout(Command::new("rustup").args(&["target", "list", "--installed"]))?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

pub fn uninstall(targets: Vec<String>) {
    println!("⏳ uninstalling targets");

    let status = Command::new("rustup")
        .args(&["target", "remove"])
        .args(&targets)
        .status()
        .unwrap();
    if !status.success() {
        // only print uninstall errors so the user can fix those manually if needed
        eprintln!("Error uninstalling targets: {}", targets.join(" "));
    }
}
