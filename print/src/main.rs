use std::{env, fmt::Error, fs, io::{self, Read}, path::PathBuf};

use anyhow::anyhow;
use defmt_decoder::{Table, elf2table::VersionError};
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

    // let table = Table::parse(&bytes)?.ok_or_else(|| anyhow!(".defmt data not found"))?;
    let table = Table::parse(&bytes);
    if let Err(ref e) = table {
        if e.is::<VersionError>() {
            println!("Version Error");
            println!("Error e: {}", e);
            // what should happen here?
        } else {
            anyhow!(".defmt data not found");
        }
    };

    let table = table?.unwrap();
    
    let locs = table.get_locations(&bytes)?;
    let locs = if table.indices().all(|idx| locs.contains_key(&(idx as u64))) {
        Some(locs)
    } else {
        log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
        None
    };

    let mut buf = [0; READ_BUFFER_SIZE];
    let mut frames = vec![];

    let current_dir = env::current_dir()?;
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    loop {
        let n = stdin.read(&mut buf)?;

        frames.extend_from_slice(&buf[..n]);

        loop {
            match table.decode(&frames) {
                Ok((frame, consumed)) => {
                    // NOTE(`[]` indexing) all indices in `table` have already been
                    // verified to exist in the `locs` map
                    let loc = locs.as_ref().map(|locs| &locs[&frame.index()]);

                    let (mut file, mut line, mut mod_path) = (None, None, None);
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

                    // Forward the defmt frame to our logger.
                    defmt_decoder::log::log_defmt(
                        &frame,
                        file.as_deref(),
                        line,
                        mod_path.as_deref(),
                    );

                    let num_frames = frames.len();
                    frames.rotate_left(consumed);
                    frames.truncate(num_frames - consumed);
                }
                Err(defmt_decoder::DecodeError::UnexpectedEof) => break,
                Err(defmt_decoder::DecodeError::Malformed) => {
                    log::error!("failed to decode defmt data: {:x?}", frames);
                    return Err(defmt_decoder::DecodeError::Malformed.into());
                }
            }
        }
    }
}

/// Report version from Cargo.toml _(e.g. "0.1.4")_ and supported `defmt`-versions.
///
/// Used by `--version` flag.
#[allow(clippy::unnecessary_wraps)]
fn print_version() -> anyhow::Result<()> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("gettin here!!!!");
    println!("supported defmt version: {}", defmt_decoder::DEFMT_VERSION);
    Ok(())
}
