use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use defmt_decoder::{
    log::{
        format::{Formatter, FormatterConfig, HostFormatter},
        DefmtLoggerType,
    },
    DecodeError, Frame, Locations, Table, DEFMT_VERSIONS,
};

use tokio::{
    fs,
    io::{self, AsyncReadExt, Stdin},
    net::TcpStream,
};

/// Prints defmt-encoded logs to stdout
#[derive(Parser)]
#[command(name = "defmt-print")]
struct Opts {
    #[arg(short, required = true, conflicts_with("version"))]
    elf: Option<PathBuf>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    log_format: Option<String>,

    #[arg(long)]
    host_log_format: Option<String>,

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
    /// Read defmt frames from stdin (default)
    Stdin,
    /// Read defmt frames from tcp
    Tcp {
        #[arg(long, env = "RTT_HOST", default_value = "localhost")]
        host: String,

        #[arg(long, env = "RTT_PORT", default_value_t = 19021)]
        port: u16,
    },
}

enum Source {
    Stdin(Stdin),
    Tcp(TcpStream),
}

impl Source {
    fn stdin() -> Self {
        Source::Stdin(io::stdin())
    }

    async fn tcp(host: String, port: u16) -> anyhow::Result<Self> {
        match TcpStream::connect((host, port)).await {
            Ok(stream) => Ok(Source::Tcp(stream)),
            Err(e) => Err(anyhow!(e)),
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<(usize, bool)> {
        match self {
            Source::Stdin(stdin) => {
                let n = stdin.read(buf).await?;
                Ok((n, n == 0))
            }
            Source::Tcp(tcpstream) => Ok((tcpstream.read(buf).await?, false)),
        }
    }
}

const READ_BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Opts {
        elf,
        json,
        log_format,
        host_log_format,
        show_skipped_frames,
        verbose,
        version,
        command,
    } = Opts::parse();

    if version {
        return print_version();
    }

    // read and parse elf file
    let bytes = fs::read(elf.unwrap()).await?;
    let table = Table::parse(&bytes)?.ok_or_else(|| anyhow!(".defmt data not found"))?;
    let locs = table.get_locations(&bytes)?;

    // check if the locations info contains all the indicies
    let locs = if table.indices().all(|idx| locs.contains_key(&(idx as u64))) {
        Some(locs)
    } else {
        log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
        None
    };

    let logger_type = if json {
        DefmtLoggerType::Json
    } else {
        DefmtLoggerType::Stdout
    };

    let cloned_format = log_format.clone().unwrap_or_default();
    let mut formatter_config = if log_format.is_some() {
        FormatterConfig::custom(cloned_format.as_str())
    } else if verbose {
        FormatterConfig::default().with_location()
    } else {
        FormatterConfig::default()
    };

    formatter_config.is_timestamp_available = table.has_timestamp();

    let cloned_host_format = host_log_format.clone().unwrap_or_default();
    let host_formatter_config = if host_log_format.is_some() {
        FormatterConfig::custom(cloned_host_format.as_str())
    } else if verbose {
        FormatterConfig::default().with_location()
    } else {
        FormatterConfig::default()
    };

    let formatter = Formatter::new(formatter_config);
    let host_formatter = HostFormatter::new(host_formatter_config);

    defmt_decoder::log::init_logger(formatter, host_formatter, logger_type, move |metadata| {
        match verbose {
            false => defmt_decoder::log::is_defmt_frame(metadata), // We display *all* defmt frames, but nothing else.
            true => true,                                          // We display *all* frames.
        }
    });

    let mut buf = [0; READ_BUFFER_SIZE];
    let mut stream_decoder = table.new_stream_decoder();
    let current_dir = env::current_dir()?;

    let mut source = match command {
        None | Some(Command::Stdin) => Source::stdin(),
        Some(Command::Tcp { host, port }) => Source::tcp(host, port).await?,
    };

    loop {
        // read from stdin or tcpstream and push it to the decoder
        let (n, eof) = source.read(&mut buf).await?;

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
    let s = if DEFMT_VERSIONS.len() > 1 { "s" } else { "" };
    println!("supported defmt version{s}: {}", DEFMT_VERSIONS.join(", "));
    Ok(())
}
