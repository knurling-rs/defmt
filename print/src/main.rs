use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
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
use goblin::elf::Elf;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt, Stdin},
    net::TcpStream,
    select,
    sync::mpsc::Receiver,
};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

/// Prints defmt-encoded logs to stdout
#[derive(Parser, Clone)]
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

    /// Tell Segger J-Link what the RTT address is
    #[arg(long)]
    set_addr: bool,

    #[arg(long)]
    show_skipped_frames: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short = 'V', long)]
    version: bool,

    #[arg(short, long)]
    watch_elf: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Clone)]
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
    Serial {
        #[arg(long, env = "SERIAL_PORT", default_value = "/dev/ttyUSB0")]
        path: PathBuf,

        #[arg(long, env = "SERIAL_BAUD", default_value_t = 115200)]
        baud: u32,

        #[arg(long, env = "SERIAL_DTR", default_value_t = false)]
        dtr: bool,
    },
}

enum Source {
    Stdin(Stdin),
    Tcp(TcpStream),
    Serial(SerialStream),
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

    fn serial(path: PathBuf, baud: u32, dtr: bool) -> anyhow::Result<Self> {
        let mut ser = tokio_serial::new(path.to_string_lossy(), baud).open_native_async()?;
        ser.set_timeout(Duration::from_millis(500))?;
        if dtr {
            ser.write_data_terminal_ready(true)?;
        }
        Ok(Source::Serial(ser))
    }

    async fn set_rtt_addr(&mut self, elf_bytes: &[u8]) -> anyhow::Result<()> {
        let Source::Tcp(tcpstream) = self else {
            return Ok(());
        };

        let elf = Elf::parse(elf_bytes)?;
        let rtt_symbol = elf
            .syms
            .iter()
            .find(|sym| elf.strtab.get_at(sym.st_name) == Some("_SEGGER_RTT"))
            .ok_or_else(|| anyhow!("Symbol '_SEGGER_RTT' not found in ELF file"))?;

        let cmd = format!(
            "$$SEGGER_TELNET_ConfigStr=SetRTTAddr;{:#x}$$",
            rtt_symbol.st_value
        );
        tcpstream.write_all(cmd.as_bytes()).await?;

        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<(usize, bool)> {
        match self {
            Source::Stdin(stdin) => {
                let n = stdin.read(buf).await?;
                Ok((n, n == 0))
            }
            Source::Tcp(tcpstream) => Ok((tcpstream.read(buf).await?, false)),
            Source::Serial(serial) => Ok((serial.read(buf).await?, false)),
        }
    }
}

const READ_BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    if opts.version {
        return print_version();
    }

    // We create the source outside of the run command since recreating the stdin looses us some frames
    let mut source = match opts.command.clone() {
        None | Some(Command::Stdin) => Source::stdin(),
        Some(Command::Tcp { host, port }) => Source::tcp(host, port).await?,
        Some(Command::Serial { path, baud, dtr }) => Source::serial(path, baud, dtr)?,
    };

    if opts.watch_elf {
        run_and_watch(opts, &mut source).await
    } else {
        run(opts, &mut source).await
    }
}

async fn has_file_changed(rx: &mut Receiver<Result<Event, notify::Error>>, path: &PathBuf) -> bool {
    loop {
        if let Some(Ok(event)) = rx.recv().await {
            if event.paths.contains(path) {
                if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                    break;
                }
            }
        }
    }
    true
}

async fn run_and_watch(opts: Opts, source: &mut Source) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let path = opts.elf.clone().unwrap().canonicalize().unwrap();

    // We want the elf directory instead of the elf, since some editors remove
    // and recreate the file on save which will remove the notifier
    let directory_path = path.parent().unwrap();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )?;
    watcher.watch(directory_path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        select! {
            r = run(opts.clone(), source) => r?,
            _ = has_file_changed(&mut rx, &path) => ()
        }
    }
}

async fn run(opts: Opts, source: &mut Source) -> anyhow::Result<()> {
    let Opts {
        elf,
        json,
        log_format,
        host_log_format,
        set_addr,
        show_skipped_frames,
        verbose,
        ..
    } = opts;

    // read and parse elf file
    let bytes = fs::read(elf.unwrap()).await?;
    let table = Table::parse(&bytes)?.ok_or_else(|| anyhow!(".defmt data not found"))?;
    let locs = table.get_locations(&bytes)?;

    if set_addr {
        // Using Segger RTT server, set the _SEGGER_RTT address actively.
        source.set_rtt_addr(&bytes).await?;
    }

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
