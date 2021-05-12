use std::{collections::HashSet, process::Command};

use crate::{run_capturing_stdout, run_command};

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

    let installed_targets = get_installed()?;
    let added_targets = required_targets
        .difference(&installed_targets)
        .cloned()
        .collect::<Vec<_>>();

    if !added_targets.is_empty() {
        println!("⏳ installing targets");

        let mut args = vec!["target", "add"];
        args.extend(added_targets.iter().map(|s| s.as_str()));
        let status = Command::new("rustup").args(&args).status().unwrap();
        if !status.success() {
            // since installing targets is the first thing we do, hard panic is OK enough (user would notice at this point)
            panic!("Error installing targets: {}", added_targets.join(" "));
        }
    }

    Ok(added_targets)
}

fn get_installed() -> anyhow::Result<HashSet<String>> {
    let stdout = run_capturing_stdout(Command::new("rustup").args(&["target", "list"]))?;

    const INSTALLED_MARKER: &str = " (installed)";
    let targets = stdout
        .lines()
        .filter(|target| target.contains(INSTALLED_MARKER))
        .map(|target| target.replace(INSTALLED_MARKER, ""))
        .collect::<HashSet<_>>();
    Ok(targets)
}

pub fn uninstall(targets: Vec<String>) {
    println!("⏳ uninstalling targets");

    let mut cmd_and_args = vec!["rustup", "target", "remove"];
    cmd_and_args.extend(targets.iter().map(|s| s.as_str()));

    // only print uninstall errors so the user can fix those manually if needed
    run_command(&cmd_and_args, None, &[])
        .unwrap_or_else(|e| eprintln!("Error uninstalling targets {}: {}", targets.join(" "), e));
}
