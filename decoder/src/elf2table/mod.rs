//! Reads ELF metadata and builds a table containing [`defmt`](https://github.com/knurling-rs/defmt) format strings.
//!
//! This is an implementation detail of [`probe-run`](https://github.com/knurling-rs/probe-run) and
//! not meant to be consumed by other tools at the moment so all the API is unstable.

mod symbol;

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    fmt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, ensure};
use object::{Object, ObjectSection, ObjectSymbol};

use crate::{BitflagsKey, StringEntry, Table, TableEntry, Tag, DEFMT_VERSIONS};

pub fn parse_impl(elf: &[u8], check_version: bool) -> Result<Option<Table>, anyhow::Error> {
    let elf = object::File::parse(elf)?;
    // first pass to extract the `_defmt_version`
    let mut version = None;
    let mut encoding = None;

    // Note that we check for a quoted and unquoted version symbol, since LLD has a bug that
    // makes it keep the quotes from the linker script.
    let try_get_version = |name: &str| {
        if name.starts_with("\"_defmt_version_ = ") || name.starts_with("_defmt_version_ = ") {
            Some(
                name.trim_start_matches("\"_defmt_version_ = ")
                    .trim_start_matches("_defmt_version_ = ")
                    .trim_end_matches('"')
                    .to_string(),
            )
        } else {
            None
        }
    };

    // No need to remove quotes for `_defmt_encoding_`, since it's defined in Rust code
    // using `#[export_name = "_defmt_encoding_ = x"]`, never in linker scripts.
    let try_get_encoding = |name: &str| {
        name.strip_prefix("_defmt_encoding_ = ")
            .map(ToString::to_string)
    };

    for entry in elf.symbols() {
        let name = match entry.name() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Not in the `.defmt` section because it's not tied to the address of any symbol
        // in `.defmt`.
        if let Some(new_version) = try_get_version(name) {
            if let Some(version) = version {
                return Err(anyhow!(
                    "multiple defmt versions in use: {} and {} (only one is supported)",
                    version,
                    new_version
                ));
            }
            version = Some(new_version);
        }

        if let Some(new_encoding) = try_get_encoding(name) {
            if let Some(encoding) = encoding {
                return Err(anyhow!(
                    "multiple defmt encodings in use: {} and {} (only one is supported)",
                    encoding,
                    new_encoding
                ));
            }
            encoding = Some(new_encoding);
        }
    }

    // NOTE: We need to make sure to return `Ok(None)`, not `Err`, when defmt is not in use.
    // Otherwise probe-run won't work with apps that don't use defmt.

    let defmt_section = elf.section_by_name(".defmt");

    let (defmt_section, version) = match (defmt_section, version) {
        (None, None) => return Ok(None), // defmt is not used
        (Some(defmt_section), Some(version)) => (defmt_section, version),
        (None, Some(_)) => {
            bail!("defmt version found, but no `.defmt` section - check your linker configuration");
        }
        (Some(_), None) => {
            bail!(
                "`.defmt` section found, but no version symbol - check your linker configuration"
            );
        }
    };

    if check_version {
        self::check_version(&version).map_err(anyhow::Error::msg)?;
    }

    let encoding = match encoding {
        Some(e) => e.parse()?,
        None => bail!("No defmt encoding specified. This is a bug."),
    };

    // second pass to demangle symbols
    let mut map = BTreeMap::new();
    let mut bitflags_map = HashMap::new();
    let mut timestamp = None;
    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            continue;
        };

        if name.is_empty() {
            // Skipping symbols with empty string names, as they may be added by
            // `objcopy`, and breaks JSON demangling
            continue;
        }

        if name == "$d" || name.starts_with("$d.") {
            // Skip AArch64 mapping symbols
            continue;
        }

        if name.starts_with("_defmt") || name.starts_with("__DEFMT_MARKER") {
            // `_defmt_version_` is not a JSON encoded `defmt` symbol / log-message; skip it
            // LLD and GNU LD behave differently here. LLD doesn't include `_defmt_version_`
            // (defined in a linker script) in the `.defmt` section but GNU LD does.
            continue;
        }

        if entry.section_index() == Some(defmt_section.index()) {
            let sym = symbol::Symbol::demangle(name)?;
            match sym.tag() {
                symbol::SymbolTag::Defmt(Tag::Timestamp) => {
                    if timestamp.is_some() {
                        bail!("multiple timestamp format specifications found");
                    }

                    timestamp = Some(TableEntry::new(
                        StringEntry::new(Tag::Timestamp, sym.data().to_string()),
                        name.to_string(),
                    ));
                }
                symbol::SymbolTag::Defmt(Tag::BitflagsValue) => {
                    // Bitflags values always occupy 128 bits / 16 bytes.
                    const BITFLAGS_VALUE_SIZE: u64 = 16;

                    if entry.size() != BITFLAGS_VALUE_SIZE {
                        bail!(
                            "bitflags value does not occupy 16 bytes (symbol `{}`)",
                            name
                        );
                    }

                    let defmt_data = defmt_section.data()?;
                    let addr = entry.address() as usize;
                    let value = match defmt_data.get(addr..addr + 16) {
                        Some(bytes) => u128::from_le_bytes(bytes.try_into().unwrap()),
                        None => bail!(
                            "bitflags value at {:#x} outside of defmt section",
                            entry.address()
                        ),
                    };
                    log::debug!("bitflags value `{}` has value {:#x}", sym.data(), value);

                    let segments = sym.data().split("::").collect::<Vec<_>>();
                    let (bitflags_name, value_idx, value_name) = match &*segments {
                        [bitflags_name, value_idx, value_name] => {
                            (*bitflags_name, value_idx.parse::<u128>()?, *value_name)
                        }
                        _ => bail!("malformed bitflags value string '{}'", sym.data()),
                    };

                    let key = BitflagsKey {
                        ident: bitflags_name.into(),
                        package: sym.package().into(),
                        disambig: sym.disambiguator().into(),
                        crate_name: sym.crate_name().map(|s| s.into()),
                    };

                    bitflags_map.entry(key).or_insert_with(Vec::new).push((
                        value_name.into(),
                        value_idx,
                        value,
                    ));
                }
                symbol::SymbolTag::Defmt(tag) => {
                    map.insert(
                        entry.address() as usize,
                        TableEntry::new(
                            StringEntry::new(tag, sym.data().to_string()),
                            name.to_string(),
                        ),
                    );
                }
                symbol::SymbolTag::Custom(_) => {}
            }
        }
    }

    // Sort bitflags values by the value's index in definition order. Since all values get their own
    // symbol and section, their order in the final binary is unspecified and can't be relied on, so
    // we put them back in the original order here.
    let bitflags = bitflags_map
        .into_iter()
        .map(|(k, mut values)| {
            values.sort_by_key(|(_, index, _)| *index);
            let values = values
                .into_iter()
                .map(|(name, _index, value)| (name, value))
                .collect();

            (k, values)
        })
        .collect();

    Ok(Some(Table {
        entries: map,
        timestamp,
        bitflags,
        encoding,
    }))
}

/// Checks if the version encoded in the symbol table is compatible with this version of the `decoder` crate
fn check_version(version: &str) -> Result<(), String> {
    if !DEFMT_VERSIONS.contains(&version) {
        let msg = format!(
            "defmt wire format version mismatch: firmware is using {}, `probe-run` supports {}\nsuggestion: use a newer version of `defmt` or `cargo install` a different version of `probe-run` that supports defmt {}",
            version, DEFMT_VERSIONS.join(", "), version
        );

        return Err(msg);
    }

    Ok(())
}

/// Location of a defmt log statement in the elf-file
#[derive(Clone)]
pub struct Location {
    pub file: PathBuf,
    pub line: u64,
    pub module: String,
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Mapping of memory address to [`Location`]
pub type Locations = BTreeMap<u64, Location>;

pub fn get_locations(elf: &[u8], table: &Table) -> Result<Locations, anyhow::Error> {
    let object = object::File::parse(elf)?;
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

    let dwarf_sections =
        gimli::DwarfSections::<Cow<[u8]>>::load::<_, anyhow::Error>(&load_section)?;
    let dwarf_sup_sections = gimli::DwarfSections::load::<_, anyhow::Error>(&load_section_sup)?;

    let borrow_section: &dyn for<'a> Fn(
        &'a Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(section, endian);

    let dwarf = dwarf_sections.borrow_with_sup(&dwarf_sup_sections, &borrow_section);

    let mut units = dwarf.debug_info.units();

    let mut map = BTreeMap::new();
    while let Some(header) = units.next()? {
        let unit = dwarf.unit(header)?;
        let abbrev = header.abbreviations(&dwarf.debug_abbrev)?;

        let mut cursor = header.entries(&abbrev);

        ensure!(cursor.next_dfs()?.is_some(), "empty DWARF?");

        let mut segments = vec![];
        let mut depth = 0;
        while let Some((delta_depth, entry)) = cursor.next_dfs()? {
            depth += delta_depth;

            // NOTE .. here start the custom logic
            if entry.tag() == gimli::constants::DW_TAG_namespace {
                let mut attrs = entry.attrs();

                while let Some(attr) = attrs.next()? {
                    if attr.name() == gimli::constants::DW_AT_name {
                        if let gimli::AttributeValue::DebugStrRef(off) = attr.value() {
                            let s = dwarf.string(off)?;
                            for _ in (depth as usize)..segments.len() + 1 {
                                segments.pop();
                            }
                            segments.push(core::str::from_utf8(&s)?.to_string());
                        }
                    }
                }
            } else if entry.tag() == gimli::constants::DW_TAG_variable {
                // Iterate over the attributes in the DIE.
                let mut attrs = entry.attrs();

                // what we are after
                let mut decl_file = None;
                let mut decl_line = None; // line number
                let mut name = None;
                let mut linkage_name = None;
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
                        gimli::constants::DW_AT_linkage_name => {
                            if let gimli::AttributeValue::DebugStrRef(off) = attr.value() {
                                linkage_name = Some(off);
                            }
                        }
                        _ => {}
                    }
                }

                if let (
                    Some(name_index),
                    Some(linkage_name_index),
                    Some(file_index),
                    Some(line),
                    Some(loc),
                ) = (name, linkage_name, decl_file, decl_line, location)
                {
                    let name_slice = dwarf.string(name_index)?;
                    let name = core::str::from_utf8(&name_slice)?;
                    let linkage_name_slice = dwarf.string(linkage_name_index)?;
                    let linkage_name = core::str::from_utf8(&linkage_name_slice)?;

                    if name == "DEFMT_LOG_STATEMENT" {
                        if table.raw_symbols().any(|i| i == linkage_name) {
                            let addr = exprloc2address(unit.encoding(), &loc)?;
                            let file = file_index_to_path(file_index, &unit, &dwarf)?;
                            let module = segments.join("::");

                            let loc = Location { file, line, module };

                            if let Some(old) = map.insert(addr, loc.clone()) {
                                bail!("BUG in DWARF variable filter: index collision for addr 0x{:08x} (old = {:?}, new = {:?})", addr, old, loc);
                            }
                        } else {
                            // this symbol was GC-ed by the linker (but remains in the DWARF info)
                            // so we discard it (its `addr` info is also wrong which causes collisions)
                        }
                    }
                }
            }
        }
    }

    Ok(map)
}

fn file_index_to_path<R>(
    index: u64,
    unit: &gimli::Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> Result<PathBuf, anyhow::Error>
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

    let mut p = PathBuf::new();
    if let Some(dir) = file.directory(header) {
        let dir = dwarf.attr_string(unit, dir)?;
        let dir_s = dir.to_string_lossy()?;
        let dir = Path::new(&dir_s[..]);

        if !dir.is_absolute() {
            if let Some(ref comp_dir) = unit.comp_dir {
                p.push(&comp_dir.to_string_lossy()?[..]);
            }
        }
        p.push(dir);
    }

    p.push(
        &dwarf
            .attr_string(unit, file.path_name())?
            .to_string_lossy()?[..],
    );

    Ok(p)
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
