use std::{
    collections::{HashMap, HashSet},
    process::{Command, Stdio},
};
use std::{env, fs};
use std::{io::Read, path::Path};

use console::Style;
use similar::{ChangeTag, TextDiff};

use anyhow::{anyhow, Context, Error, Result};

fn load_expected_output(name: &str, release_mode: bool) -> Result<String> {
    let path = Path::new("firmware/qemu/src/bin");

    let filename;
    if release_mode {
        filename = format!("{}.release.out", name);
    } else {
        filename = format!("{}.out", name);
    }

    let path = path.join(filename);

    // for error context closure
    let path_str = path.to_str().unwrap_or("(non-Unicode path)").to_owned();

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to load expected output data from {}", path_str))?;
    Ok(content)
}

fn run_returning_stdout(cmd: &mut Command) -> Result<String> {
    let child = cmd.stdout(Stdio::piped()).spawn()?;
    let mut stdout = child
        .stdout
        .ok_or(anyhow!("could not access standard output"))?;
    let mut out = String::new();
    stdout
        .read_to_string(&mut out)
        .with_context(|| "could not read standard output")?;
    Ok(out)
}

fn test_qemu(name: &str, features: &str, release_mode: bool) -> Result<()> {
    let display_name = format!(
        "{} ({})",
        name,
        if release_mode { "release" } else { "dev" }
    );
    println!("{}", display_name);
    let cwd_name = "firmware/qemu".to_owned();
    let cwd = fs::canonicalize(&cwd_name)
        .map_err(|e| anyhow!("running {} in {}: {}", display_name, cwd_name, e))?;
    let mut args;
    if release_mode {
        args = vec!["-q", "rrb", name]
    } else {
        args = vec!["-q", "rb", name]
    }
    if features.len() > 0 {
        args.extend_from_slice(&["--features", features]);
    }

    let actual = run_returning_stdout(Command::new("cargo").args(&args).current_dir(cwd))?;

    let expected = load_expected_output(name, release_mode)?;

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
        Err(anyhow!("{}", display_name))
    }
}

fn run_test<F>(t: F, context: &str, errors: &mut Vec<String>) -> ()
where
    F: FnOnce() -> Result<()>,
{
    match t() {
        Ok(_) => {}
        Err(e) => errors.push(format!("{}: {}", context, e)),
    }
}

fn rustc_is_nightly() -> Result<bool> {
    let out = run_returning_stdout(Command::new("rustc").args(&["-V"]))?;
    Ok(out.contains("nightly"))
}

fn test_snapshot(errors: &mut Vec<String>) -> () {
    println!("*** qemu/snapshot ***");
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

    match rustc_is_nightly() {
        Ok(is_nightly) => {
            if is_nightly {
                tests.push("alloc");
            }
        }
        Err(e) => {
            eprintln!(
                "could not determine whether rust compiler is nightly - assuming it's not ({})",
                e
            )
        }
    }

    let mut features_map = HashMap::new();
    features_map.insert("alloc", "alloc");
    let no_features = "";

    for test in &tests {
        let features = features_map.get(test).unwrap_or(&no_features);

        run_test(|| test_qemu(test, features, false), "qemu/snapshot", errors);
        run_test(|| test_qemu(test, features, true), "qemu/snapshot", errors);
    }
}

fn get_installed_targets() -> Result<HashSet<String>> {
    let out = run_returning_stdout(Command::new("rustup").args(&["target", "list"]))?;
    let installed_marker = " (installed)";
    let mut targets = out.lines().collect::<Vec<_>>();
    targets.retain(|target| target.contains(installed_marker));
    let targets: HashSet<String> = targets
        .iter()
        .map(|target| target.replace(installed_marker, ""))
        .collect();
    Ok(targets)
}

fn install_targets() -> Result<Vec<String>> {
    println!("installing targets");
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

    let installed_targets = get_installed_targets()?;
    let missing_targets = all_targets.difference(&installed_targets);
    let mut added_targets = vec![];

    // since installing targets is the first thing we do, hard panic is fine
    for target in missing_targets {
        let status = Command::new("rustup")
            .args(&["target", "add", target])
            .status()
            .unwrap();
        if !status.success() {
            panic!("Error installing target: {}", target);
        }
        added_targets.push(target.to_owned());
    }

    Ok(added_targets)
}

fn uninstall_targets(targets: Vec<String>) {
    println!("uninstalling targets");

    // print all uninstall errors so the user can fix those manually if needed
    for target in targets {
        match run_command(&["rustup", "target", "remove", &target]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error uninstalling target {}: {}", target, e);
            }
        }
    }
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let subcommand = args.next();
    match subcommand.as_deref() {
        Some("test_all") => {
            let keep_targets = match args.next().as_deref() {
                Some("-k") => true,
                _ => false,
            };
            let added_targets = match install_targets() {
                Ok(targets) => targets,
                Err(e) => {
                    panic!("Error while installing required targets: {}", e)
                }
            };

            let mut all_errors: Vec<String> = vec![];

            test_host(&mut all_errors);
            test_cross(&mut all_errors);
            test_snapshot(&mut all_errors);
            test_book(&mut all_errors);
            test_lint(&mut all_errors);

            if !all_errors.is_empty() {
                eprintln!("");
                eprintln!("some tests failed:");
                for error in all_errors {
                    eprintln!("{}", error);
                }
            }
            if !keep_targets {
                uninstall_targets(added_targets);
            }
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

fn test_lint(all_errors: &mut Vec<String>) -> () {}

fn test_book(all_errors: &mut Vec<String>) -> () {}

fn run_command(cmd_and_args: &[&str]) -> Result<()> {
    let cmd_and_args = Vec::from(cmd_and_args);
    let mut cmd = Command::new(cmd_and_args[0]);
    if cmd_and_args.len() > 1 {
        cmd.args(&cmd_and_args[1..]);
    }

    let cmdline = cmd_and_args.join(" ");
    cmd.status()
        .map_err(|e| anyhow!("could not run '{}': {}", cmdline, e))
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                "'{}' did not finish successfully (non-zero exit status or killed by signal): {:?}",
                cmdline,
                exit_status.code()
            ))
            }
        })
}

fn test_cross(errors: &mut Vec<String>) -> () {
    println!("*** cross ***");
    let targets = vec![
        "thumbv6m-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];

    for target in targets {
        run_test(
            || run_command(&["cargo", "check", "--target", &target, "-p", "defmt"]),
            "cross",
            errors,
        );
        run_test(
            || {
                run_command(&[
                    "cargo",
                    "check",
                    "--target",
                    &target,
                    "-p",
                    "defmt",
                    "--features",
                    "alloc",
                ])
            },
            "cross",
            errors,
        );
    }

    //cargo check --target $target -p defmt
    //cargo check --target $target -p defmt --features alloc
}

fn test_host(all_errors: &mut Vec<String>) -> () {}
