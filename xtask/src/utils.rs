use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::{self, FromStr},
};

use anyhow::{anyhow, Context};
use colored::Colorize;

pub fn load_expected_output(name: &str, is_test: bool) -> anyhow::Result<String> {
    let path = expected_output_path(name, is_test);

    fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to load expected output data from {}",
            path.to_str().unwrap_or("(non-Unicode path)")
        )
    })
}

pub fn overwrite_expected_output(name: &str, contents: &[u8], is_test: bool) -> anyhow::Result<()> {
    let file = expected_output_path(name, is_test);
    let path = Path::new(&file);

    fs::write(path, contents).with_context(|| {
        format!(
            "Failed to overwrite expected output data to {}",
            path.to_str().unwrap_or("(non-Unicode path)")
        )
    })
}

fn expected_output_path(name: &str, is_test: bool) -> PathBuf {
    const PROJECT_DIR: &str = "firmware/qemu";

    let mut path = PathBuf::from(PROJECT_DIR);
    if is_test {
        path.push("tests")
    } else {
        path.push("src");
        path.push("bin");
    };
    path.push(name);
    path.set_extension("out");
    path
}

/// Execute the [`Command`]. If success return `stdout`, if failure print to `stderr`
pub fn run_capturing_stdout(cmd: &mut Command) -> anyhow::Result<String> {
    let output = cmd.output()?;
    match output.status.success() {
        true => Ok(str::from_utf8(&output.stdout)?.to_string()),
        false => {
            eprintln!("{}", str::from_utf8(&output.stderr)?.dimmed());
            Err(anyhow!(""))
        }
    }
}

pub fn run_command(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
) -> anyhow::Result<()> {
    run_command_with_stdout(program, args, cwd, envs, Stdio::inherit()).map(|_| ())
}

pub fn run_command_with_stdout(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
    stdout: Stdio,
) -> anyhow::Result<String> {
    let mut cmd = Command::new(program);
    cmd.args(args).envs(envs.iter().copied());
    cmd.stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(stdout);
    let cwd = if let Some(path) = cwd {
        cmd.current_dir(path);
        format!("{path}$ ")
    } else {
        "".to_string()
    };

    let cmdline = format!("{cwd}{program} {}", args.join(" "));
    println!("🏃 {cmdline}");

    cmd.output()
        .map_err(|e| anyhow!("could not run '{}': {}", cmdline, e))
        .and_then(|output| match output.status.success() {
            true => String::from_utf8(output.stdout).context("stdout was not valid UTF-8"),
            false => Err(anyhow!(
                "'{}' did not finish successfully: {}",
                cmdline,
                output.status
            )),
        })
}

/// Build a target and return all paths that contain related cargo build artifacts
pub fn build_library_dirs(args: &[&str], cwd: Option<&str>) -> anyhow::Result<BTreeSet<String>> {
    let mut build_args = vec!["build", "--message-format=json-render-diagnostics"];
    build_args.extend_from_slice(args);
    let output = run_command_with_stdout("cargo", &build_args, cwd, &[], Stdio::piped())?;

    let mut library_dirs = BTreeSet::new();
    for line in output.lines() {
        let message = serde_json::Value::from_str(line)?;
        if message["reason"] != "compiler-artifact" {
            continue;
        }
        let filenames = message["filenames"]
            .as_array()
            .context("Cargo artifact missing filenames")?;
        for filename in filenames {
            let path = Path::new(filename.as_str().context("Cargo artifact filename invalid")?);
            let parent = path
                .parent()
                .context("Cargo artifact missing parent")?
                .to_str()
                .context("Cargo artifact directory not UTF-8")?
                .to_owned();
            library_dirs.insert(parent);
        }
    }

    Ok(library_dirs)
}

pub fn rustc_is_nightly() -> bool {
    // if this crashes the system is not in a good state, so we'll not pretend to be able to recover
    let out = run_capturing_stdout(Command::new("rustc").args(["-V"])).unwrap();
    out.contains("nightly")
}

pub fn rustc_is_msrv() -> bool {
    const MSRV: &str = "1.81";
    let out = run_capturing_stdout(Command::new("rustc").args(["-V"])).unwrap();
    out.contains(MSRV)
}
