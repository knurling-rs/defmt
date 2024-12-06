use crate::do_test;
use crate::utils::{load_expected_output, run_capturing_stdout};
use anyhow::{anyhow, Context};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::process::{Command, Stdio};

pub fn test_print_snapshot(name: &str) {
    do_test(
        || {
            println!("{}", name.bold());

            let frame_path = format!("xtask/output_files/{}.out", name);
            let elf_path = format!("xtask/snapshot_elfs/{}", name);

            let frames = Command::new("cat")
                .arg(frame_path)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let actual = run_capturing_stdout(
                Command::new("defmt-print")
                    .arg("-e")
                    .arg(elf_path)
                    .stdin(Stdio::from(frames.stdout.unwrap())),
            )
            .with_context(|| name.to_string())?;

            let expected = load_expected_output(name, false)?;
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
        },
        "qemu/print_snapshot",
    )
}
