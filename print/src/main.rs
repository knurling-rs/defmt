use std::{
    env, fs,
    io::{self, Read, Stdin, StdinLock},
    net::TcpStream,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error};
use clap::{Parser, Subcommand};
use defmt_decoder::{DecodeError, Frame, Locations, Table, DEFMT_VERSIONS};

/// Prints defmt-encoded logs to stdout
#[derive(Parser)]
#[command(name = "defmt-print")]
struct Opts {
    #[arg(short, required = true, conflicts_with("version"))]
    elf: Option<PathBuf>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    show_skipped_frames: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short = 'V', long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Read defmt frames from stdin
    Stdin,
    /// Read defmt frames from tcp
    Tcp {
        #[arg(long, env = "RTT_HOST", default_value = "127.0.0.1")]
        host: String,

        #[arg(long, env = "RTT_PORT", default_value_t = 19021)]
        port: u16,
    },
}

enum Source<'a> {
    Stdin(StdinLock<'a>),
    Tcp(TcpStream),
}

const READ_BUFFER_SIZE: usize = 1024;

fn main() -> anyhow::Result<()> {
    let Opts {
        elf,
        json,
        show_skipped_frames,
        verbose,
        version,
        command,
    } = Opts::parse();

    if version {
        return print_version();
    }

    defmt_decoder::log::init_logger(verbose, json, move |metadata| match verbose {
        false => defmt_decoder::log::is_defmt_frame(metadata), // We display *all* defmt frames, but nothing else.
        true => true,                                          // We display *all* frames.
    });

    let bytes = fs::read(elf.unwrap())?;

    let table = Table::parse(&bytes)?.ok_or_else(|| anyhow!(".defmt data not found"))?;
    let locs = table.get_locations(&bytes)?;

    let locs = if table.indices().all(|idx| locs.contains_key(&(idx as u64))) {
        Some(locs)
    } else {
        log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
        None
    };

    let mut buf = [0; READ_BUFFER_SIZE];
    let mut stream_decoder = table.new_stream_decoder();

    let current_dir = env::current_dir()?;
    let stdin = io::stdin();

    let mut source: Source = match command {
        None => open_stdin(&stdin),
        Some(Command::Stdin) => open_stdin(&stdin),
        Some(Command::Tcp { host, port }) => open_tcp(host, port),
    }?;

    loop {
        // read from stdin or tcpstream and push it to the decoder
        let (n, eof) = match source {
            Source::Stdin(ref mut stdin) => {
                let n = stdin.read(&mut buf)?;
                (n, n == 0)
            }
            Source::Tcp(ref mut tcpstream) => (tcpstream.read(&mut buf)?, false),
        };

        // if 0 bytes where read, we reached EOF, so quit
        if eof {
            break Ok(());
        }

        stream_decoder.received(&buf[..n]);

        // decode the received data
        loop {
            match stream_decoder.decode() {
                Ok(frame) => forward_to_logger(&frame, location_info(&locs, &frame, &current_dir)),
                Err(DecodeError::UnexpectedEof) => break,
                Err(DecodeError::Malformed) => match table.encoding().can_recover() {
                    // if recovery is impossible, abort
                    false => return Err(DecodeError::Malformed.into()),
                    // if recovery is possible, skip the current frame and continue with new data
                    true => {
                        // bug: https://github.com/rust-lang/rust-clippy/issues/9810
                        #[allow(clippy::print_literal)]
                        if show_skipped_frames || verbose {
                            println!("(HOST) malformed frame skipped");
                            println!("└─ {} @ {}:{}", env!("CARGO_PKG_NAME"), file!(), line!());
                        }
                        continue;
                    }
                },
            }
        }
    }
}

fn open_stdin(stdin: &Stdin) -> Result<Source, Error> {
    Ok(Source::Stdin(stdin.lock()))
}

fn open_tcp<'a>(host: String, port: u16) -> Result<Source<'a>, Error> {
    match TcpStream::connect(format!("{host}:{port}")) {
        Ok(stream) => Ok(Source::Tcp(stream)),
        Err(e) => Err(anyhow!(e)),
    }
}

type LocationInfo = (Option<String>, Option<u32>, Option<String>);

fn forward_to_logger(frame: &Frame, location_info: LocationInfo) {
    let (file, line, mod_path) = location_info;
    defmt_decoder::log::log_defmt(frame, file.as_deref(), line, mod_path.as_deref());
}

fn location_info(locs: &Option<Locations>, frame: &Frame, current_dir: &Path) -> LocationInfo {
    let (mut file, mut line, mut mod_path) = (None, None, None);

    let loc = locs.as_ref().map(|locs| locs.get(&frame.index()));

    if let Some(Some(loc)) = loc {
        // try to get the relative path, else the full one
        let path = loc.file.strip_prefix(current_dir).unwrap_or(&loc.file);

        file = Some(path.display().to_string());
        line = Some(loc.line as u32);
        mod_path = Some(loc.module.clone());
    }

    (file, line, mod_path)
}

/// Report version from Cargo.toml _(e.g. "0.1.4")_ and supported `defmt`-versions.
///
/// Used by `--version` flag.
#[allow(clippy::unnecessary_wraps)]
fn print_version() -> anyhow::Result<()> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!(
        "supported defmt version{}: {}",
        if DEFMT_VERSIONS.len() > 1 { "s" } else { "" },
        DEFMT_VERSIONS.join(", ")
    );
    Ok(())
}
