use std::{
    env, fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::anyhow;
use defmt_decoder::{DecodeError, Encoding, Frame, Locations, Table};
use structopt::StructOpt;

/// Prints defmt-encoded logs to stdout
#[derive(StructOpt)]
#[structopt(name = "defmt-print")]
struct Opts {
    #[structopt(short, parse(from_os_str), required_unless_one(&["version"]))]
    elf: Option<PathBuf>,

    #[structopt(short = "V", long)]
    version: bool,
    // may want to add this later
    // #[structopt(short, long)]
    // verbose: bool,
    // TODO add file path argument; always use stdin for now
}

const READ_BUFFER_SIZE: usize = 1024;

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    if opts.version {
        return print_version();
    }

    let verbose = false;
    defmt_decoder::log::init_logger(verbose, |metadata| {
        // We display *all* defmt frames, but nothing else.
        defmt_decoder::log::is_defmt_frame(metadata)
    });

    let bytes = fs::read(&opts.elf.unwrap())?;

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
    let mut stdin = stdin.lock();

    loop {
        // read from stdin and push it to the decoder
        let n = stdin.read(&mut buf)?;
        stream_decoder.received(&buf[..n]);

        // decode the received data
        loop {
            match stream_decoder.decode() {
                Ok(frame) => {
                    let location_info = obtain_location_info(&locs, &frame, &current_dir);
                    forward_defmt_frame_to_logger(&frame, location_info);
                }
                Err(DecodeError::UnexpectedEof) => break,
                Err(DecodeError::Malformed) => match table.encoding() {
                    // raw encoding doesn't allow for recovery. therefore we abort.
                    Encoding::Raw => return Err(DecodeError::Malformed.into()),
                    // rzcobs encoding allows recovery from decoding-errors. therefore we stop
                    // decoding the current, corrupted, data and continue with new one
                    Encoding::Rzcobs => break,
                },
            }
        }
    }
}

type LocationInfo = (Option<String>, Option<u32>, Option<String>);

fn obtain_location_info(
    locs: &Option<Locations>,
    frame: &Frame,
    current_dir: &PathBuf,
) -> LocationInfo {
    let (mut file, mut line, mut mod_path) = (None, None, None);

    // NOTE(`[]` indexing) all indices in `table` have been verified to exist in the `locs` map
    let loc = locs.as_ref().map(|locs| &locs[&frame.index()]);

    if let Some(loc) = loc {
        let relpath = if let Ok(relpath) = loc.file.strip_prefix(&current_dir) {
            relpath
        } else {
            // not relative; use full path
            &loc.file
        };
        file = Some(relpath.display().to_string());
        line = Some(loc.line as u32);
        mod_path = Some(loc.module.clone());
    }

    (file, line, mod_path)
}

fn forward_defmt_frame_to_logger(frame: &Frame, location_info: LocationInfo) {
    let (file, line, mod_path) = location_info;
    defmt_decoder::log::log_defmt(&frame, file.as_deref(), line, mod_path.as_deref());
}

/// Report version from Cargo.toml _(e.g. "0.1.4")_ and supported `defmt`-versions.
///
/// Used by `--version` flag.
#[allow(clippy::unnecessary_wraps)]
fn print_version() -> anyhow::Result<()> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("supported defmt version: {}", defmt_decoder::DEFMT_VERSION);
    Ok(())
}
