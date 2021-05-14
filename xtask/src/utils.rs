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

pub fn run_command(cmd_and_args: &[&str], cwd: Option<&str>, env: &[(&str, &str)]) -> anyhow::Result<()> {
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

pub fn rustc_is_nightly() -> bool {
    // if this crashes the system is not in a good state, so we'll not pretend to be able to recover
    let out = run_capturing_stdout(Command::new("rustc").args(&["-V"])).unwrap();
    out.contains("nightly")
}
