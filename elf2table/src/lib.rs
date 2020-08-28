//! Reads ELF metadata and builds an interner table

use std::{borrow::Cow, collections::BTreeMap, path::PathBuf};

use anyhow::{anyhow, bail, ensure};
pub use decoder::Table;
use object::{Object, ObjectSection};

/// Parses an ELF file and returns the decoded `defmt` table
///
/// This function returns `None` if the ELF file contains no `.defmt` section
pub fn parse(elf: &object::File) -> Result<Option<Table>, anyhow::Error> {
    // find the index of the `.defmt` section
    let defmt_shndx = if let Some(section) = elf.section_by_name(".defmt") {
        section.index()
    } else {
        return Ok(None);
    };

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

        // Not in the `.defmt` section because it's not tied to the address of any symbol
        // in `.defmt`.
        // Note that we check for a quoted and unquoted version symbol, since LLD has a bug that
        // makes it keep the quotes from the linker script.
        if name.starts_with("\"_defmt_version_ = ") || name.starts_with("_defmt_version_ = ") {
            let new_version = name
                .trim_start_matches("\"_defmt_version_ = ")
                .trim_start_matches("_defmt_version_ = ")
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

#[derive(Debug)]
pub struct Location {
    pub file: PathBuf,
    pub line: u64,
}

pub type Locations = BTreeMap<u64, Location>;

pub fn get_locations(object: &object::File) -> Result<Locations, anyhow::Error> {
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let load_section = |id: gimli::SectionId| {
        Ok(if let Some(s) = object.section_by_name(id.name()) {
            s.uncompressed_data().unwrap_or(Cow::Borrowed(&[][..]))
        } else {
            Cow::Borrowed(&[][..])
        })
    };
    let load_section_sup = |_| Ok(Cow::Borrowed(&[][..]));

    let dwarf_cow =
        gimli::Dwarf::<Cow<[u8]>>::load::<_, _, anyhow::Error>(&load_section, &load_section_sup)?;

    let borrow_section: &dyn for<'a> Fn(
        &'a Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    let dwarf = dwarf_cow.borrow(&borrow_section);

    let mut units = dwarf.debug_info.units();

    let mut map = BTreeMap::new();
    while let Some(header) = units.next()? {
        let unit = dwarf.unit(header)?;
        let abbrev = header.abbreviations(&dwarf.debug_abbrev)?;

        let mut cursor = header.entries(&abbrev);

        ensure!(cursor.next_dfs()?.is_some(), "empty DWARF?");

        while let Some((_, entry)) = cursor.next_dfs()? {
            // NOTE .. here start the custom logic
            if entry.tag() == gimli::constants::DW_TAG_variable {
                // Iterate over the attributes in the DIE.
                let mut attrs = entry.attrs();

                // what we are after
                let mut decl_file = None;
                let mut decl_line = None; // line number
                let mut name = None;
                let mut location = None;

                while let Some(attr) = attrs.next()? {
                    match attr.name() {
                        gimli::constants::DW_AT_name => {
                            if let gimli::AttributeValue::DebugStrRef(off) = attr.value() {
                                name = Some(off);
                            }
                        }

                        gimli::constants::DW_AT_decl_file => {
                            if let gimli::AttributeValue::FileIndex(idx) = attr.value() {
                                decl_file = Some(idx);
                            }
                        }

                        gimli::constants::DW_AT_decl_line => {
                            if let gimli::AttributeValue::Udata(line) = attr.value() {
                                decl_line = Some(line);
                            }
                        }

                        gimli::constants::DW_AT_location => {
                            if let gimli::AttributeValue::Exprloc(loc) = attr.value() {
                                location = Some(loc);
                            }
                        }

                        _ => {}
                    }
                }

                if name.is_some()
                    && decl_file.is_some()
                    && decl_line.is_some()
                    && location.is_some()
                {
                    if let (Some(name_index), Some(file_index), Some(line), Some(loc)) =
                        (name, decl_file, decl_line, location)
                    {
                        let endian_slice = dwarf.string(name_index)?;
                        let name = core::str::from_utf8(&endian_slice)?;

                        if name == "DEFMT_LOG_STATEMENT" {
                            let addr = exprloc2address(unit.encoding(), &loc)?;
                            let file =
                                PathBuf::from(file_index_to_string(file_index, &unit, &dwarf)?);

                            let loc = Location { file, line };

                            if addr != 0 {
                                ensure!(
                                    map.insert(addr, loc).is_none(),
                                    "BUG in DWARF variable filter: index collision"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(map)
}

fn file_index_to_string<R>(
    index: u64,
    unit: &gimli::Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> Result<String, anyhow::Error>
where
    R: gimli::read::Reader,
{
    ensure!(index != 0, "`FileIndex` was zero");

    let header = if let Some(program) = &unit.line_program {
        program.header()
    } else {
        bail!("no `LineProgram`");
    };

    let file = if let Some(file) = header.file(index) {
        file
    } else {
        bail!("no `FileEntry` for index {}", index)
    };

    let mut s = String::new();
    if let Some(dir) = file.directory(header) {
        let dir = dwarf.attr_string(unit, dir)?;
        let dir = dir.to_string_lossy()?;

        if !dir.starts_with('/') {
            if let Some(ref comp_dir) = unit.comp_dir {
                s.push_str(&comp_dir.to_string_lossy()?);
                s.push('/');
            }
        }
        s.push_str(&dir);
        s.push('/');
    }

    s.push_str(
        &dwarf
            .attr_string(unit, file.path_name())?
            .to_string_lossy()?,
    );

    Ok(s)
}

fn exprloc2address<R: gimli::read::Reader<Offset = usize>>(
    encoding: gimli::Encoding,
    data: &gimli::Expression<R>,
) -> Result<u64, anyhow::Error> {
    let mut pc = data.0.clone();
    while pc.len() != 0 {
        if let Ok(gimli::Operation::Address { address }) =
            gimli::Operation::parse(&mut pc, encoding)
        {
            return Ok(address);
        }
    }

    Err(anyhow!("`Operation::Address` not found"))
}
