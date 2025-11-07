//! # qemu-run
//!
//! An alternative to the [`probe-run`](https://github.com/knurling-rs/probe-run) printer,
//! used by [`defmt`](https://github.com/knurling-rs/defmt).
//!
//! Parses data sent by QEMU over semihosting (ARM Cortex-M only).

use std::{
    env, fs,
    io::Read as _,
    process::{self, Command, Stdio},
};

use anyhow::{anyhow, bail};
use clap::Parser;
use defmt_decoder::{DecodeError, StreamDecoder, Table};
use process::Child;

/// Run qemu-system-arm, takes defmt logs from semihosting output and prints them to stdout
#[derive(clap::Parser, Clone)]
#[command(name = "qemu-run")]
struct Opts {
    /// The firmware running on the device being logged
    #[arg(required = false, conflicts_with("version"))]
    elf: Option<std::path::PathBuf>,

    /// Specify the QEMU machine type
    #[arg(long, required = false)]
    machine: Option<String>,

    /// Specify the QEMU CPU type
    #[arg(long, required = false)]
    cpu: Option<String>,

    /// Print the version number, and quit
    #[arg(short = 'V', long)]
    version: bool,

    /// Print the version number, and quit
    #[arg(short = 'v', long)]
    verbose: bool,
}

fn main() -> Result<(), anyhow::Error> {
    notmain().map(|opt_code| {
        if let Some(code) = opt_code {
            process::exit(code);
        }
    })
}

fn notmain() -> Result<Option<i32>, anyhow::Error> {
    let opts = Opts::parse();

    if opts.version {
        return print_version();
    }

    let Some(elf_path) = opts.elf else {
        bail!("ELF filename is required. Syntax: `qemu-run -machine <machine> <path-to-elf>`.");
    };

    let Some(machine) = opts.machine else {
        bail!("Machine type is required. Syntax: `qemu-run -machine <machine> <path-to-elf>`.");
    };

    if opts.verbose {
        eprintln!("QEMU machine is {:?}", machine);
        if let Some(cpu) = &opts.cpu {
            eprintln!("QEMU cpu is {:?}", cpu);
        }
    }

    let bytes = fs::read(&elf_path)?;

    let table = if env::var_os("QEMU_RUN_IGNORE_VERSION").is_some() {
        Table::parse_ignore_version(&bytes)
    } else {
        Table::parse(&bytes)
    };
    let table = match table {
        Ok(Some(table)) => table,
        Ok(None) => {
            bail!("Loaded ELF but did not find a .defmt section");
        }
        Err(e) => {
            bail!("Failed to load ELF: {:?}", e);
        }
    };

    let mut command = Command::new("qemu-system-arm");
    command.args([
        "-machine",
        &machine,
        "-nographic",
        "-monitor",
        "none",
        "-semihosting-config",
        "enable=on,target=native",
        "-kernel",
    ]);
    command.arg(elf_path);
    command.stdout(Stdio::piped());

    if let Some(cpu) = &opts.cpu {
        command.arg("-cpu");
        command.arg(cpu);
    }

    if opts.verbose {
        eprintln!("Running: {:?}", command);
    }

    let mut child = KillOnDrop(
        command
            .spawn()
            .expect("Error running qemu-system-arm; perhaps you haven't installed it yet?"),
    );

    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to acquire child's stdout handle"))?;

    let mut decoder = table.new_stream_decoder();

    let mut readbuf = [0; 256];
    let exit_code;
    loop {
        let n = stdout.read(&mut readbuf)?;
        decoder.received(&readbuf[..n]);
        decode_and_print(decoder.as_mut())?;

        if let Some(status) = child.0.try_wait()? {
            // process finished - grab all remaining bytes and quit
            exit_code = status.code();
            let mut data = Vec::new();
            stdout.read_to_end(&mut data)?;
            decoder.received(&data);
            decode_and_print(decoder.as_mut())?;
            break;
        }
    }

    // pass back qemu exit code (if any)
    Ok(exit_code)
}

/// Pump the decoder and print any new frames
fn decode_and_print(decoder: &mut dyn StreamDecoder) -> Result<(), DecodeError> {
    loop {
        match decoder.decode() {
            Ok(frame) => {
                println!("{}", frame.display(true))
            }
            Err(DecodeError::UnexpectedEof) => return Ok(()),
            Err(DecodeError::Malformed) => {
                eprintln!("failed to decode defmt data");
                return Err(DecodeError::Malformed);
            }
        }
    }
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        self.0.kill().ok();
    }
}

/// Report version from Cargo.toml _(e.g. "0.1.4")_ and supported `defmt`-versions.
///
/// Used by `--version` flag.
#[allow(clippy::unnecessary_wraps)]
fn print_version() -> anyhow::Result<Option<i32>> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let s = if defmt_decoder::DEFMT_VERSIONS.len() > 1 {
        "s"
    } else {
        ""
    };
    println!(
        "supported defmt version{s}: {}",
        defmt_decoder::DEFMT_VERSIONS.join(", ")
    );
    Ok(Some(0))
}
