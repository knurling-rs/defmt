//! Reads ELF metadata and builds an interner table

use std::collections::BTreeMap;

use anyhow::anyhow;
pub use decoder::Table;
use object::{File, Object, ObjectSection};

/// Parses an ELF file and returns the decoded `defmt` table
///
/// This function returns `None` if the ELF file contains no `.defmt` section
pub fn parse(elf: &[u8]) -> Result<Option<Table>, anyhow::Error> {
    let elf = File::parse(elf)?;

    // find the index of the `.defmt` section
    let defmt_shndx = elf
        .section_by_name(".defmt")
        .ok_or_else(|| anyhow!("`.defmt` section is missing"))?
        .index();

    let mut map = BTreeMap::new();
    let mut version = None;
    let mut trace_start = None;
    let mut trace_end = None;
    let mut debug_start = None;
    let mut debug_end = None;
    let mut info_start = None;
    let mut info_end = None;
    let mut warn_start = None;
    let mut warn_end = None;
    let mut error_start = None;
    let mut error_end = None;
    for (_, entry) in elf.symbols() {
        let name = match entry.name() {
            Some(name) => name,
            None => continue,
        };

        // not in the `.defmt` section because it's not tied to the address of any symbol
        // in `.defmt`
        if name.starts_with("\"_defmt_version_ = ") {
            let new_version = name
                .trim_start_matches("\"_defmt_version_ = ")
                .trim_end_matches('"');
            if let Some(version) = version {
                return Err(anyhow!(
                    "multiple defmt versions in use: {} and {} (only one is supported)",
                    version,
                    new_version
                ));
            }
            version = Some(new_version);
        }

        if entry.section_index() == Some(defmt_shndx) {
            match name {
                "_defmt_trace_start" => trace_start = Some(entry.address() as usize),
                "_defmt_trace_end" => trace_end = Some(entry.address() as usize),
                "_defmt_debug_start" => debug_start = Some(entry.address() as usize),
                "_defmt_debug_end" => debug_end = Some(entry.address() as usize),
                "_defmt_info_start" => info_start = Some(entry.address() as usize),
                "_defmt_info_end" => info_end = Some(entry.address() as usize),
                "_defmt_warn_start" => warn_start = Some(entry.address() as usize),
                "_defmt_warn_end" => warn_end = Some(entry.address() as usize),
                "_defmt_error_start" => error_start = Some(entry.address() as usize),
                "_defmt_error_end" => error_end = Some(entry.address() as usize),
                _ => {
                    map.insert(entry.address() as usize, name.to_string());
                }
            }
        }
    }

    // unify errors
    let (error, warn, info, debug, trace, version) = (|| -> Option<_> {
        Some((
            error_start?..error_end?,
            warn_start?..warn_end?,
            info_start?..info_end?,
            debug_start?..debug_end?,
            trace_start?..trace_end?,
            version?,
        ))
    })()
    .ok_or_else(|| anyhow!("`_defmt_*` symbol not found"))?;

    Table::new(map, debug, error, info, trace, warn, version)
        .map_err(anyhow::Error::msg)
        .map(Some)
}
