use std::{
    collections::{HashMap, HashSet},
    process::{Command, Stdio},
};
use std::{env, fs};
use std::{io::Read, path::Path};

use console::Style;
use similar::{ChangeTag, TextDiff};

fn load_expected_output(name: &str, release_mode: bool) -> String {
    let path = Path::new("firmware/qemu/src/bin");

    let filename;
    if release_mode {
        filename = format!("{}.release.out", name);
    } else {
        filename = format!("{}.out", name);
    }

    let path = path.join(filename);

    fs::read_to_string(path).unwrap()
}

fn run_returning_stdout(cmd: &mut Command) -> String {
    let mut cmd = cmd.stdout(Stdio::piped()).spawn().unwrap();
    let mut stdout = cmd.stdout.take().unwrap();
    let mut out = String::new();
    stdout.read_to_string(&mut out).unwrap();
    out
}

fn rustc_is_nightly() -> bool {
    let out = run_returning_stdout(Command::new("rustc").args(&["-V"]));
    out.contains("nightly")
}

fn run_qemu(name: &str, features: &str, release_mode: bool) -> Result<(), String> {
    let display_name = format!(
        "{} ({})",
        name,
        if release_mode { "release" } else { "dev" }
    );
    println!("testing {}", display_name,);
    let cwd = fs::canonicalize("firmware/qemu").unwrap();
    let mut args;
    if release_mode {
        args = vec!["-q", "rrb", name]
    } else {
        args = vec!["-q", "rb", name]
    }
    if features.len() > 0 {
        args.extend_from_slice(&["--features", features]);
    }

    let AAA = 1;

    let actual = run_returning_stdout(Command::new("cargo").args(&args).current_dir(cwd));

    let expected = load_expected_output(name, release_mode);

    let diff = TextDiff::from_lines(&expected, &actual);

    // if anything isn't ChangeTag::Equal, print it and turn on error flag
    let mut actual_matches_expected = true;
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let styled_change = match change.tag() {
                ChangeTag::Delete => Some(("-", Style::new().red())),
                ChangeTag::Insert => Some(("+", Style::new().green())),
                ChangeTag::Equal => None,
            };
            match styled_change {
                Some((sign, style)) => {
                    actual_matches_expected = false;
                    eprint!("{}{}", style.apply_to(sign).bold(), style.apply_to(change),);
                }
                None => {}
            }
        }
    }

    if actual_matches_expected {
        Ok(())
    } else {
        eprintln!("ERROR");
        Err(display_name)
    }
}

fn test_all_qemu() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut tests = vec![
        "log",
        "timestamp",
        "panic",
        "assert",
        "assert-eq",
        "assert-ne",
        "unwrap",
        "defmt-test",
        "hints",
    ];

    if rustc_is_nightly() {
        tests.push("alloc");
    }

    let mut features_map = HashMap::new();
    features_map.insert("alloc", "alloc");
    let no_features = "";

    for test in &tests {
        let features = features_map.get(test).unwrap_or(&no_features);

        match run_qemu(test, features, false) {
            Ok(_) => {}
            Err(e) => errors.push(e),
        }

        match run_qemu(test, features, true) {
            Ok(_) => {}
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn get_installed_targets() -> HashSet<String> {
    let out = run_returning_stdout(Command::new("rustup").args(&["target", "list"]));
    let installed_marker = " (installed)";
    let mut targets = out.lines().collect::<Vec<_>>();
    targets.retain(|target| target.contains(installed_marker));
    let targets = targets
        .iter()
        .map(|target| target.replace(installed_marker, ""))
        .collect();
    targets
}

fn install_targets() -> Vec<String> {
    let required_targets = vec![
        "thumbv6m-none-eabi",
        "thumbv7m-none-eabi",
        "thumbv7em-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];
    let all_targets = required_targets
        .iter()
        .map(|item| item.to_string())
        .collect::<HashSet<_>>();

    let installed_targets = get_installed_targets();
    let missing_targets = all_targets.difference(&installed_targets);
    let mut added_targets = vec![];

    for target in missing_targets {
        Command::new("rustup")
            .args(&["target", "add", target])
            .spawn()
            .unwrap()
            .wait()
            .ok();
        added_targets.push(target.to_owned());
    }

    added_targets
}

fn uninstall_targets(targets: Vec<String>) {
    for target in targets {
        Command::new("rustup")
            .args(&["target", "remove", &target])
            .spawn()
            .unwrap()
            .wait()
            .ok();
    }
}

// TODO currently there's a lot of unwrap() going on, meaning if anything breaks
// the targets installed for this run will not be uninstalled again

// TODO cmdline flag for keeping installed targets
fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let subcommand = args.next();
    match subcommand.as_deref() {
        Some("test_all") => {
            let added_targets = install_targets();
            uninstall_targets(added_targets);
            test_all_qemu().map_err(|e| format!("Some tests failed: {}", e.join(", ")))?;
            Ok(())
        }
        _ => {
            eprintln!("usage: cargo xtask <subcommand>");
            eprintln!();
            eprintln!("subcommands:");
            eprintln!("    test_all - run all tests");
            Err("".into())
        }
    }
}
