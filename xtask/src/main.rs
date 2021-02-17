use std::{
    collections::{HashMap, HashSet},
    fs,
    process::{Command, Stdio},
};
use std::{io::Read, path::Path};

use console::Style;
use similar::{ChangeTag, TextDiff};

use anyhow::{anyhow, Context, Result};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(
        long,
        short,
        help = "keep target toolchains that were installed as dependency"
    )]
    keep_targets: bool,
    #[structopt(subcommand)]
    cmd: TestCommand,
}

#[derive(StructOpt, Debug)]
enum TestCommand {
    TestAll,
    TestHost,
    TestCross,
    TestSnapshot,
    TestBook,
    TestLint,
}

fn run_command<P: AsRef<Path>>(cmd_and_args: &[&str], cwd: Option<P>) -> Result<()> {
    let cmd_and_args = Vec::from(cmd_and_args);
    let mut cmd = &mut Command::new(cmd_and_args[0]);
    if cmd_and_args.len() > 1 {
        cmd.args(&cmd_and_args[1..]);
    }

    let cwd_s: String;
    match cwd {
        Some(path_ref) => {
            cmd = cmd.current_dir(path_ref.as_ref());

            cwd_s = format!(
                "{}{}",
                path_ref
                    .as_ref()
                    .to_str()
                    .unwrap_or("(non-Unicode path)")
                    .to_string(),
                std::path::MAIN_SEPARATOR
            );
        }
        None => {
            cwd_s = "".to_string();
        }
    }

    let cmdline = cmd_and_args.join(" ");
    cmd.status()
        .map_err(|e| anyhow!("could not run '{}{}': {}", cwd_s, cmdline, e))
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                let info = match exit_status.code() {
                    Some(code) => {
                        format!("non-zero exit status: {}", code)
                    }
                    None => "killed by signal".to_string(),
                };

                Err(anyhow!(
                    "'{}' did not finish successfully: {}",
                    cmdline,
                    info
                ))
            }
        })
}

fn run_capturing_stdout(cmd: &mut Command) -> Result<String> {
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

fn do_test<F>(t: F, context: &str, errors: &mut Vec<String>)
where
    F: FnOnce() -> Result<()>,
{
    match t() {
        Ok(_) => {}
        Err(e) => errors.push(format!("{}: {}", context, e)),
    }
}

fn rustc_is_nightly() -> Result<bool> {
    let out = run_capturing_stdout(Command::new("rustc").args(&["-V"]))?;
    Ok(out.contains("nightly"))
}

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
    let path_str = path.to_str().unwrap_or("(non-Unicode path)").to_string();

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to load expected output data from {}", path_str))?;
    Ok(content)
}

fn test_single_snapshot(name: &str, features: &str, release_mode: bool) -> Result<()> {
    let display_name = format!(
        "{} ({})",
        name,
        if release_mode { "release" } else { "dev" }
    );
    println!("{}", display_name);
    let cwd_name = "firmware/qemu".to_string();
    let mut args;
    if release_mode {
        args = vec!["-q", "rrb", name]
    } else {
        args = vec!["-q", "rb", name]
    }
    if !features.is_empty() {
        args.extend_from_slice(&["--features", features]);
    }

    let actual = run_capturing_stdout(Command::new("cargo").args(&args).current_dir(cwd_name))?;

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

fn get_installed_targets() -> Result<HashSet<String>> {
    let out = run_capturing_stdout(Command::new("rustup").args(&["target", "list"]))?;
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
    let missing_targets: Vec<&String> = all_targets.difference(&installed_targets).collect();
    let mut added_targets: Vec<String> = vec![];

    if !missing_targets.is_empty() {
        println!("⏳ installing targets");
    }
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
    if !targets.is_empty() {
        println!("⏳ uninstalling targets");
    }

    // print all uninstall errors so the user can fix those manually if needed
    for target in targets {
        match run_command::<&str>(&["rustup", "target", "remove", &target], None) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error uninstalling target {}: {}", target, e);
            }
        }
    }
}

fn test_book(errors: &mut Vec<String>) -> () {
    println!("🧪 book");
    do_test(
        || run_command::<&str>(&["cargo", "clean"], None),
        "book",
        errors,
    );

    do_test(
        || run_command::<&str>(&["cargo", "build", "--features", "unstable-test"], None),
        "book",
        errors,
    );

    do_test(
        || {
            run_command(
                &[
                    "mdbook",
                    "test",
                    "-L",
                    "../target/debug",
                    "-L",
                    "../target/debug/deps",
                ],
                Some("book"),
            )
        },
        "book",
        errors,
    );
}

fn test_lint(errors: &mut Vec<String>) -> () {
    println!("🧪 lint");
    do_test(
        || run_command::<&str>(&["cargo", "fmt", "--all", "--", "--check"], None),
        "lint",
        errors,
    );

    do_test(
        || run_command::<&str>(&["cargo", "clippy", "--workspace"], None),
        "lint",
        errors,
    );
}

fn test_host(errors: &mut Vec<String>) -> () {
    println!("🧪 host");

    do_test(
        || {
            run_command::<&str>(
                &[
                    "cargo",
                    "test",
                    "--workspace",
                    "--features",
                    "unstable-test",
                ],
                None,
            )
        },
        "host",
        errors,
    );
}

fn test_cross(errors: &mut Vec<String>) -> () {
    println!("🧪 cross");
    let targets = vec![
        "thumbv6m-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];

    for target in targets {
        do_test(
            || {
                run_command::<&str>(
                    &["cargo", "check", "--target", &target, "-p", "defmt"],
                    None,
                )
            },
            "cross",
            errors,
        );
        do_test(
            || {
                run_command::<&str>(
                    &[
                        "cargo",
                        "check",
                        "--target",
                        &target,
                        "-p",
                        "defmt",
                        "--features",
                        "alloc",
                    ],
                    None,
                )
            },
            "cross",
            errors,
        );
    }

    do_test(
        || {
            run_command(
                &[
                    "cargo",
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--workspace",
                    "--exclude",
                    "defmt-itm",
                    "--exclude",
                    "firmware",
                ],
                Some("firmware"),
            )
        },
        "cross",
        errors,
    );

    do_test(
        || {
            run_command(
                &[
                    "cargo",
                    "check",
                    "--target",
                    "thumbv7em-none-eabi",
                    "--workspace",
                ],
                Some("firmware"),
            )
        },
        "cross",
        errors,
    );

    do_test(
        || {
            run_command(
                &[
                    "cargo",
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-defmt",
                ],
                Some("firmware/panic-probe"),
            )
        },
        "cross",
        errors,
    );

    do_test(
        || {
            run_command::<&str>(
                &[
                    "cargo",
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-rtt",
                ],
                Some("firmware/panic-probe"),
            )
        },
        "cross",
        errors,
    )
}

fn test_snapshot(errors: &mut Vec<String>) -> () {
    println!("🧪 qemu/snapshot");
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

        do_test(
            || test_single_snapshot(test, features, false),
            "qemu/snapshot",
            errors,
        );
        do_test(
            || test_single_snapshot(test, features, true),
            "qemu/snapshot",
            errors,
        );
    }
}

fn main() -> Result<(), Vec<String>> {
    let opt: Options = Options::from_args();
    let mut all_errors: Vec<String> = vec![];

    // TODO: one could argue that not all test scenarios require installation of targets
    let added_targets = match install_targets() {
        Ok(targets) => targets,
        Err(e) => {
            panic!("Error while installing required targets: {}", e)
        }
    };

    match opt.cmd {
        TestCommand::TestAll => {
            test_host(&mut all_errors);
            test_cross(&mut all_errors);
            test_snapshot(&mut all_errors);
            test_book(&mut all_errors);
            test_lint(&mut all_errors);
        }
        TestCommand::TestHost => {
            test_host(&mut all_errors);
        }
        TestCommand::TestCross => {
            test_cross(&mut all_errors);
        }
        TestCommand::TestSnapshot => {
            test_snapshot(&mut all_errors);
        }
        TestCommand::TestBook => {
            test_book(&mut all_errors);
        }
        TestCommand::TestLint => {
            test_lint(&mut all_errors);
        }
    }

    if !opt.keep_targets {
        uninstall_targets(added_targets);
    }

    if !all_errors.is_empty() {
        eprintln!();
        eprintln!("☠️ some tests failed:");
        for error in &all_errors {
            eprintln!("{}", error);
        }
        Err(all_errors)
    } else {
        Ok(())
    }
}
