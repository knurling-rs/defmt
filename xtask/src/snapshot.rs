use std::{process::Command, str::FromStr};

use anyhow::{anyhow, Context};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use crate::{
    do_test,
    utils::{load_expected_output, overwrite_expected_output, run_capturing_stdout, rustc_is_nightly},
};

pub const SNAPSHOT_TESTS_DIRECTORY: &str = "firmware/qemu";
pub const ALL_SNAPSHOT_TESTS: [&str; 12] = [
    "log",
    "bitflags",
    "timestamp",
    "panic",
    "assert",
    "assert-eq",
    "assert-ne",
    "unwrap",
    "defmt-test",
    "hints",
    "hints_inner",
    "dbg",
];

#[derive(Clone, Debug)]
pub struct Snapshot(String);

impl Snapshot {
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl FromStr for Snapshot {
    type Err = String;

    fn from_str(test: &str) -> Result<Self, Self::Err> {
        if ALL_SNAPSHOT_TESTS.contains(&test) {
            Ok(Self(String::from(test)))
        } else {
            Err(format!(
                "Specified test '{}' does not exist, available tests are: {:?}",
                test, ALL_SNAPSHOT_TESTS
            ))
        }
    }
}

pub fn test_snapshot(overwrite: bool, snapshot: Option<Snapshot>) {
    println!("🧪 qemu/snapshot");

    match snapshot {
        None => test_all_snapshots(overwrite),
        Some(snapshot) => {
            do_test(
                || test_single_snapshot(snapshot.name(), "", overwrite),
                "qemu/snapshot",
            );
        }
    }
}

fn test_all_snapshots(overwrite: bool) {
    let mut tests = ALL_SNAPSHOT_TESTS.to_vec();

    if rustc_is_nightly() {
        tests.extend(["alloc", "net"]);
    }

    for test in tests {
        let features = match test {
            "alloc" => "alloc",
            "net" => "ip_in_core",
            _ => "",
        };

        do_test(
            || test_single_snapshot(test, features, overwrite),
            "qemu/snapshot",
        );
    }
}

fn test_single_snapshot(name: &str, features: &str, overwrite: bool) -> anyhow::Result<()> {
    println!("{}", name.bold());

    let is_test = name.contains("test");

    let mut args = match is_test {
        true => vec!["-q", "tt", name],
        false => vec!["-q", "rb", name],
    };

    if !features.is_empty() {
        args.extend_from_slice(&["--features", features]);
    }

    let actual = run_capturing_stdout(
        Command::new("cargo")
            .args(&args)
            .env("DEFMT_LOG", "trace")
            .current_dir(SNAPSHOT_TESTS_DIRECTORY),
    )
    .with_context(|| name.to_string())?;

    if overwrite {
        overwrite_expected_output(name, actual.as_bytes(), is_test)?;
        return Ok(());
    }

    let expected = load_expected_output(name, is_test)?;
    let diff = TextDiff::from_lines(&expected, &actual);

    // if anything isn't ChangeTag::Equal, print it and turn on error flag
    let mut actual_matches_expected = true;
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let styled_change = match change.tag() {
                ChangeTag::Delete => Some(("-".bold().red(), change.to_string().red())),
                ChangeTag::Insert => Some(("+".bold().green(), change.to_string().green())),
                ChangeTag::Equal => None,
            };
            if let Some((sign, change)) = styled_change {
                actual_matches_expected = false;
                eprint!("{sign}{change}");
            }
        }
    }

    if actual_matches_expected {
        Ok(())
    } else {
        Err(anyhow!("{}", name))
    }
}
