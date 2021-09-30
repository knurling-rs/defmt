use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use colored::Colorize as _;
use tempfile::TempDir;

use crate::{utils, ALL_ERRORS, ALL_SNAPSHOT_TESTS, SNAPSHOT_TESTS_DIRECTORY};

// PR #564
const REVISION_UNDER_TEST: &str = "45beb423a5c2b4e6c645ea98b293513a6feadf6d";

// the target name is in `firmware/qemu/.cargo/config.toml` but it'd be hard to extract it from that file
const RUNNER_ENV_VAR: &str = "CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER";

pub fn test() {
    println!("🧪 backcompat");

    println!("building old qemu-run.. (git revision: {})", REVISION_UNDER_TEST);
    let qemu_run = match QemuRun::build() {
        Ok(qemu_run) => qemu_run,
        Err(e) => {
            // only print build errors so the user can fix those manually if needed
            eprintln!("error building old qemu-run: {}", e);
            ALL_ERRORS
                .lock()
                .unwrap()
                .push("backcompat (building qemu-run)".to_string());
            return;
        }
    };

    for release_mode in [false, true] {
        for snapshot_test in ALL_SNAPSHOT_TESTS {
            super::do_test(
                || qemu_run.run_snapshot(snapshot_test, release_mode),
                "backcompat",
            );
        }
    }
}

struct QemuRun {
    executable_path: PathBuf,
    _tempdir: TempDir,
}

impl QemuRun {
    fn build() -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;

        let tempdir_path = tempdir.path();
        clone_repo(tempdir_path)?;
        let executable_path = build_qemu_run(tempdir_path)?;

        Ok(Self {
            executable_path,
            _tempdir: tempdir,
        })
    }

    fn run_snapshot(&self, name: &str, release_mode: bool) -> anyhow::Result<()> {
        let formatted_test_name = utils::formatted_test_name(name, release_mode);
        println!("{}", formatted_test_name.bold());

        let args = if release_mode {
            ["-q", "rrb", name]
        } else {
            ["-q", "rb", name]
        };

        run_silently(
            Command::new("cargo")
                .args(args)
                .current_dir(SNAPSHOT_TESTS_DIRECTORY)
                .env(RUNNER_ENV_VAR, self.path()),
            || anyhow!("{}", formatted_test_name),
        )?;

        Ok(())
    }

    fn path(&self) -> &Path {
        &self.executable_path
    }
}

fn clone_repo(tempdir: &Path) -> anyhow::Result<()> {
    let repo_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    run_silently(
        Command::new("git")
            .arg("clone")
            .arg(repo_path)
            .arg(".")
            .current_dir(tempdir),
        || anyhow!("`git clone` failed"),
    )?;

    run_silently(
        Command::new("git")
            .args(&["reset", "--hard", REVISION_UNDER_TEST])
            .current_dir(tempdir),
        || anyhow!("`git reset` failed"),
    )?;

    Ok(())
}

fn build_qemu_run(tempdir: &Path) -> anyhow::Result<PathBuf> {
    run_silently(
        Command::new("cargo")
            .args(&["build", "-p", "qemu-run"])
            .current_dir(tempdir),
        || anyhow!("`cargo build` failed"),
    )?;

    let mut executable_path = tempdir.to_owned();
    executable_path.push("target");
    executable_path.push("debug");
    executable_path.push("qemu-run");

    assert!(executable_path.exists(), "`qemu-run` executable not found");

    Ok(executable_path)
}

fn run_silently(command: &mut Command, err: impl FnOnce() -> anyhow::Error) -> anyhow::Result<()> {
    let output = command.output()?;

    if !output.status.success() {
        let formatted_command = format!("{:?}", command);

        if !output.stdout.is_empty() {
            println!(
                "stdout:\n{}",
                std::str::from_utf8(&output.stdout).map_err(|e| anyhow!(
                    "`{}` output is not UTF-8: {}",
                    formatted_command,
                    e
                ))?
            );
        }

        if !output.stderr.is_empty() {
            println!(
                "stderr:\n{}",
                std::str::from_utf8(&output.stderr).map_err(|e| anyhow!(
                    "`{}` output is not UTF-8: {}",
                    formatted_command,
                    e
                ))?
            );
        }

        println!(
            "exit-code: {}",
            output
                .status
                .code()
                .map(|code| code.to_string().into())
                .unwrap_or(Cow::Borrowed("non-zero"))
        );

        return Err(err());
    }

    Ok(())
}
