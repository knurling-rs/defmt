use std::{
    env, fs,
    io::Read as _,
    process::{self, Command, Stdio},
};

use anyhow::{anyhow, bail};
use defmt_decoder::{DecodeError, Table};
use process::Child;

fn main() -> Result<(), anyhow::Error> {
    notmain().map(|opt_code| {
        if let Some(code) = opt_code {
            process::exit(code);
        }
    })
}

fn notmain() -> Result<Option<i32>, anyhow::Error> {
    let args = env::args().skip(1 /* program name */).collect::<Vec<_>>();

    if args.len() != 1 {
        bail!("expected exactly one argument. Syntax: `qemu-run <path-to-elf>`");
    }

    let path = &args[0];
    let bytes = fs::read(path)?;
    let table =
        defmt_elf2table::parse(&bytes)?.ok_or_else(|| anyhow!("`.defmt` section not found"))?;

    let mut child = KillOnDrop(
        Command::new("qemu-system-arm")
            .args(&[
                "-cpu",
                "cortex-m3",
                "-machine",
                "lm3s6965evb",
                "-nographic",
                "-monitor",
                "none",
                "-semihosting-config",
                "enable=on,target=native",
                "-kernel",
            ])
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Error running qemu-system-arm; perhaps you haven't installed it yet?"),
    );

    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to acquire child's stdout handle"))?;

    let mut frames = vec![];
    let mut readbuf = [0; 256];
    let exit_code;
    loop {
        let n = stdout.read(&mut readbuf)?;

        if n != 0 {
            frames.extend_from_slice(&readbuf[..n]);

            decode(&mut frames, &table)?;
        }

        if let Some(status) = child.0.try_wait()? {
            exit_code = status.code();

            stdout.read_to_end(&mut frames)?;
            decode(&mut frames, &table)?;
            if !frames.is_empty() {
                return Err(anyhow!(
                    "couldn't decode all data (remaining: {:x?})",
                    frames
                ));
            }

            break;
        }
    }

    Ok(exit_code)
}

fn decode(frames: &mut Vec<u8>, table: &Table) -> Result<(), DecodeError> {
    loop {
        match defmt_decoder::decode(&frames, &table) {
            Ok((frame, consumed)) => {
                println!("{}", frame.display(true));
                let n = frames.len();
                frames.rotate_left(consumed);
                frames.truncate(n - consumed);
            }
            Err(DecodeError::UnexpectedEof) => return Ok(()),
            Err(DecodeError::Malformed) => return Err(DecodeError::Malformed),
        }
    }
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        self.0.kill().ok();
    }
}
