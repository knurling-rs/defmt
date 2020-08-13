//! Reads ELF metadata and builds an interner table

use std::collections::BTreeMap;

use anyhow::{anyhow, bail};
pub use decoder::Table;
use xmas_elf::{sections::SectionData, symbol_table::Entry as _, ElfFile};

/// Parses an ELF file and returns the decoded `binfmt` table
///
/// This function returns `None` if the ELF file contains no `.binfmt` section
pub fn parse(elf: &ElfFile) -> Result<Option<Table>, anyhow::Error> {
    // find the index of the `.binfmt` section
    let binfmt_shndx = if let Some(shndx) = elf
        .section_iter()
        .zip(0..)
        .filter_map(|(sect, shndx)| {
            if sect.get_name(elf) == Ok(".binfmt") {
                Some(shndx)
            } else {
                None
            }
        })
        .next()
    {
        shndx
    } else {
        return Ok(None);
    };

    let symtab = elf
        .find_section_by_name(".symtab")
        .ok_or_else(|| anyhow!("`.symtab` section not found"))?;

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
    match symtab.get_data(elf).map_err(anyhow::Error::msg)? {
        // NOTE assuming 32-bit target
        SectionData::SymbolTable32(entries) => {
            for entry in entries {
                let name = entry.get_name(&elf);

                // not in the `.binfmt` section because it's not tied to the address of any symbol
                // in `.binfmt`
                if name == Ok("_binfmt_version_") {
                    version = Some(entry.value() as usize);
                }

                if entry.shndx() == binfmt_shndx {
                    let name = name.map_err(anyhow::Error::msg)?;
                    match name {
                        "_binfmt_trace_start" => trace_start = Some(entry.value() as usize),
                        "_binfmt_trace_end" => trace_end = Some(entry.value() as usize),
                        "_binfmt_debug_start" => debug_start = Some(entry.value() as usize),
                        "_binfmt_debug_end" => debug_end = Some(entry.value() as usize),
                        "_binfmt_info_start" => info_start = Some(entry.value() as usize),
                        "_binfmt_info_end" => info_end = Some(entry.value() as usize),
                        "_binfmt_warn_start" => warn_start = Some(entry.value() as usize),
                        "_binfmt_warn_end" => warn_end = Some(entry.value() as usize),
                        "_binfmt_error_start" => error_start = Some(entry.value() as usize),
                        "_binfmt_error_end" => error_end = Some(entry.value() as usize),
                        _ => {
                            map.insert(entry.value() as usize, name.to_string());
                        }
                    }
                }
            }
        }
        _ => bail!("`.symtab` section does not contain a symbol table"),
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
    .ok_or_else(|| anyhow!("`_binfmt_*` symbol not found"))?;

    Table::new(map, trace, debug, info, warn, error, version)
        .map_err(anyhow::Error::msg)
        .map(Some)
}
