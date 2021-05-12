use std::{fs, path::Path, process::Command, str, sync::Mutex};

use anyhow::{anyhow, Context};
use console::Style;
use once_cell::sync::Lazy;
use similar::{ChangeTag, TextDiff};
use structopt::StructOpt;

mod targets;

static ALL_ERRORS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(subcommand)]
    cmd: TestCommand,
    #[structopt(long, short)]
    /// Treat compiler warnings as errors (`RUSTFLAGS="--deny warnings"`)
    deny_warnings: bool,
    #[structopt(long, short)]
    /// Keep target toolchains that were installed as dependency
    keep_targets: bool,
}

#[derive(Debug, StructOpt)]
#[allow(clippy::enum_variant_names)]
enum TestCommand {
    TestAll,
    TestBook,
    TestCross,
    TestHost,
    TestLint,
    TestSnapshot,
}

fn main() -> anyhow::Result<()> {
    let opt: Options = Options::from_args();

    // TODO: one could argue that not all test scenarios require installation of targets
    let added_targets = targets::install().expect("Error while installing required targets");

    match opt.cmd {
        TestCommand::TestAll => {
            test_host(opt.deny_warnings);
            test_cross();
            test_snapshot();
            test_book();
            test_lint();
        }
        TestCommand::TestHost => test_host(opt.deny_warnings),
        TestCommand::TestCross => test_cross(),
        TestCommand::TestSnapshot => test_snapshot(),
        TestCommand::TestBook => test_book(),
        TestCommand::TestLint => test_lint(),
    }

    if !opt.keep_targets && !added_targets.is_empty() {
        targets::uninstall(added_targets);
    }

    let all_errors = ALL_ERRORS.lock().unwrap();
    if !all_errors.is_empty() {
        eprintln!();
        Err(anyhow!("😔 some tests failed: {:?}", all_errors))
    } else {
        Ok(())
    }
}

fn run_command(cmd_and_args: &[&str], cwd: Option<&str>, env: &[(&str, &str)]) -> anyhow::Result<()> {
    let cmd_and_args = Vec::from(cmd_and_args);
    let mut cmd = &mut Command::new(cmd_and_args[0]);
    if cmd_and_args.len() > 1 {
        cmd.args(&cmd_and_args[1..]);
    }

    for (k, v) in env {
        cmd.env(k, v);
    }

    let cwd_s = if let Some(path_ref) = cwd {
        cmd = cmd.current_dir(path_ref);
        format!("CWD:{} ", path_ref)
    } else {
        "".to_string()
    };

    let cmdline = cmd_and_args.join(" ");
    println!("🏃 {}{}", cwd_s, cmdline);
    cmd.status()
        .map_err(|e| anyhow!("could not run '{}{}': {}", cwd_s, cmdline, e))
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                Err(anyhow!("'{}' did not finish successfully: {}", cmdline, exit_status))
            }
        })
}

/// Execute the [`Command`] and return the text emitted to `stdout`, if valid UTF-8
fn run_capturing_stdout(cmd: &mut Command) -> anyhow::Result<String> {
    let stdout = cmd.output()?.stdout;
    Ok(str::from_utf8(&stdout)?.to_string())
}

fn do_test<F>(t: F, context: &str)
where
    F: FnOnce() -> anyhow::Result<()>,
{
    match t() {
        Ok(_) => {}
        Err(e) => ALL_ERRORS.lock().unwrap().push(format!("{}: {}", context, e)),
    }
}

fn rustc_is_nightly() -> bool {
    // if this crashes the system is not in a good state, so we'll not pretend to be able to recover
    let out = run_capturing_stdout(Command::new("rustc").args(&["-V"])).unwrap();
    out.contains("nightly")
}

fn load_expected_output(name: &str, release_mode: bool) -> anyhow::Result<String> {
    const BASE: &str = "firmware/qemu/src/bin";
    let file = match release_mode {
        true => format!("{}/{}.release.out", BASE, name),
        false => format!("{}/{}.out", BASE, name),
    };
    let path = Path::new(&file);

    fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to load expected output data from {}",
            path.to_str().unwrap_or("(non-Unicode path)")
        )
    })
}

fn test_single_snapshot(name: &str, features: &str, release_mode: bool) -> anyhow::Result<()> {
    let display_name = format!("{} ({})", name, if release_mode { "release" } else { "dev" });
    println!("{}", display_name);

    let mut args = if release_mode {
        vec!["-q", "rrb", name]
    } else {
        vec!["-q", "rb", name]
    };

    if !features.is_empty() {
        args.extend_from_slice(&["--features", features]);
    }

    const CWD: &str = "firmware/qemu";
    let actual = run_capturing_stdout(Command::new("cargo").args(&args).current_dir(CWD))?;
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
            if let Some((sign, style)) = styled_change {
                actual_matches_expected = false;
                eprint!("{}{}", style.apply_to(sign).bold(), style.apply_to(change),);
            }
        }
    }

    if actual_matches_expected {
        Ok(())
    } else {
        Err(anyhow!("{}", display_name))
    }
}

fn test_book() {
    println!("🧪 book");
    do_test(|| run_command(&["cargo", "clean"], None, &[]), "book");

    do_test(
        || run_command(&["cargo", "build", "--features", "unstable-test"], None, &[]),
        "book",
    );

    do_test(
        || {
            run_command(
                &["mdbook", "test", "-L", "../target/debug", "-L", "../target/debug/deps"],
                Some("book"),
                &[],
            )
        },
        "book",
    );
}

fn test_lint() {
    println!("🧪 lint");
    do_test(|| run_command(&["cargo", "clean"], None, &[]), "lint");
    do_test(
        || run_command(&["cargo", "fmt", "--all", "--", "--check"], None, &[]),
        "lint",
    );

    do_test(|| run_command(&["cargo", "clippy", "--workspace"], None, &[]), "lint");
}

fn test_host(deny_warnings: bool) {
    println!("🧪 host");

    let env = if deny_warnings {
        vec![("RUSTFLAGS", "--deny warnings")]
    } else {
        vec![]
    };

    do_test(|| run_command(&["cargo", "check", "--workspace"], None, &env), "host");

    do_test(
        || {
            run_command(
                &["cargo", "check", "--workspace", "--features", "unstable-test"],
                None,
                &env,
            )
        },
        "host",
    );

    do_test(
        || run_command(&["cargo", "check", "--workspace", "--features", "alloc"], None, &env),
        "host",
    );

    do_test(
        || {
            run_command(
                &["cargo", "test", "--workspace", "--features", "unstable-test"],
                None,
                &[],
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                &["cargo", "test", "--workspace", "--features", "unstable-test"],
                None,
                &[],
            )
        },
        "host",
    );
}

fn test_cross() {
    println!("🧪 cross");
    let targets = [
        "thumbv6m-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];

    for target in &targets {
        do_test(
            || run_command(&["cargo", "check", "--target", target, "-p", "defmt"], None, &[]),
            "cross",
        );
        do_test(
            || {
                run_command(
                    &[
                        "cargo",
                        "check",
                        "--target",
                        target,
                        "-p",
                        "defmt",
                        "--features",
                        "alloc",
                    ],
                    None,
                    &[],
                )
            },
            "cross",
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
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                &["cargo", "check", "--target", "thumbv7em-none-eabi", "--workspace"],
                Some("firmware"),
                &[],
            )
        },
        "cross",
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
                &[],
            )
        },
        "cross",
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
                    "print-rtt",
                ],
                Some("firmware/panic-probe"),
                &[],
            )
        },
        "cross",
    )
}

fn test_snapshot() {
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

    if rustc_is_nightly() {
        tests.push("alloc");
    }

    for test in tests {
        let features = if test == "alloc" { "alloc" } else { "" };

        do_test(|| test_single_snapshot(test, features, false), "qemu/snapshot");
        do_test(|| test_single_snapshot(test, features, true), "qemu/snapshot");
    }
}
