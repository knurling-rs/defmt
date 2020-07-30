use std::{
    env, fs,
    io::Read as _,
    process::{self, Command, Stdio},
};

use anyhow::{anyhow, bail};
use xmas_elf::ElfFile;

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
    let elf = ElfFile::new(&bytes).map_err(anyhow::Error::msg)?;
    let table = elf2table::parse(&elf)?;

    let mut child = Command::new("qemu-system-arm")
        .args(&[
            "-cpu",
            "cortex-m3",
            "-machine",
            "lm3s6965evb",
            "-nographic",
            "-semihosting-config",
            "enable=on,target=native",
            "-kernel",
        ])
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to acquire child's stdout handle"))?;

    let mut frames = vec![];
    let mut readbuf = [0; 256];
    let mut exit_code = None;
    let mut exited = false;
    loop {
        let n = stdout.read(&mut readbuf)?;

        if n != 0 {
            frames.extend_from_slice(&readbuf[..n]);

            if let Ok((frame, consumed)) = decoder::decode(&frames, &table) {
                println!("{}", frame.display(true));
                let n = frames.len();
                frames.rotate_left(consumed);
                frames.truncate(n - consumed);
            }
        }

        if exited {
            break;
        }

        if let Some(status) = child.try_wait()? {
            exit_code = status.code();
            // try to read once more before breaking out from this loop
            exited = true;
        }
    }

    Ok(exit_code)
}
