use std::{fs, path::Path, process::Command, str};

use anyhow::{anyhow, Context};
use colored::Colorize;

pub fn load_expected_output(name: &str, release_mode: bool) -> anyhow::Result<String> {
    let file = expected_output_path(name, release_mode);
    let path = Path::new(&file);

    fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to load expected output data from {}",
            path.to_str().unwrap_or("(non-Unicode path)")
        )
    })
}

pub fn overwrite_expected_output(name: &str, release_mode: bool, contents: &[u8]) -> anyhow::Result<()> {
    let file = expected_output_path(name, release_mode);
    let path = Path::new(&file);

    fs::write(path, contents).with_context(|| {
        format!(
            "Failed to overwrite expected output data to {}",
            path.to_str().unwrap_or("(non-Unicode path)")
        )
    })
}

fn expected_output_path(name: &str, release_mode: bool) -> String {
    const BASE: &str = "firmware/qemu/src/bin";
    match release_mode {
        true => format!("{}/{}.release.out", BASE, name),
        false => format!("{}/{}.out", BASE, name),
    }
}

/// Execute the [`Command`]. If success return `stdout`, if failure print to `stderr`
pub fn run_capturing_stdout(cmd: &mut Command) -> anyhow::Result<String> {
    let output = cmd.output()?;
    match output.status.success() {
        true => Ok(str::from_utf8(&output.stdout)?.to_string()),
        false => {
            // #[should_error]-tests return non-zero code
            if output.status.code() == Some(1) {
                let stdout = str::from_utf8(&output.stdout)?;
                if stdout.contains(
                    "`#[should_error]` test failed with outcome: Ok(this should have returned `Err`)",
                ) {
                    return Ok(stdout.to_string());
                }
            }
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
    let mut cmd = Command::new(program);
    cmd.args(args).envs(envs.iter().copied());

    let cwd = if let Some(path) = cwd {
        cmd.current_dir(path);
        format!("{}$ ", path)
    } else {
        "".to_string()
    };

    let cmdline = format!("{}{} {}", cwd, program, args.join(" "));
    println!("🏃 {}", cmdline);

    cmd.status()
        .map_err(|e| anyhow!("could not run '{}': {}", cmdline, e))
        .and_then(|exit_status| match exit_status.success() {
            true => Ok(()),
            false => Err(anyhow!(
                "'{}' did not finish successfully: {}",
                cmdline,
                exit_status
            )),
        })
}

pub fn rustc_is_nightly() -> bool {
    // if this crashes the system is not in a good state, so we'll not pretend to be able to recover
    let out = run_capturing_stdout(Command::new("rustc").args(&["-V"])).unwrap();
    out.contains("nightly")
}
