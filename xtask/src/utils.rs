use std::{fs, path::Path, process::Command, str};

use anyhow::{anyhow, Context};

pub fn load_expected_output(name: &str, release_mode: bool) -> anyhow::Result<String> {
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

/// Execute the [`Command`] and return the text emitted to `stdout`, if valid UTF-8
pub fn run_capturing_stdout(cmd: &mut Command) -> anyhow::Result<String> {
    let stdout = cmd.output()?.stdout;
    Ok(str::from_utf8(&stdout)?.to_string())
}

pub fn run_command(program: &str, args: &[&str], cwd: Option<&str>, envs: &[(&str, &str)]) -> anyhow::Result<()> {
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
            false => Err(anyhow!("'{}' did not finish successfully: {}", cmdline, exit_status)),
        })
}

pub fn rustc_is_nightly() -> bool {
    // if this crashes the system is not in a good state, so we'll not pretend to be able to recover
    let out = run_capturing_stdout(Command::new("rustc").args(&["-V"])).unwrap();
    out.contains("nightly")
}
