//! Reads ELF metadata and builds a table containing [`defmt`](https://github.com/knurling-rs/defmt) format strings.
//!
//! This is an implementation detail of [`probe-run`](https://github.com/knurling-rs/probe-run) and
//! not meant to be consumed by other tools at the moment so all the API is unstable.

mod symbol;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    convert::TryInto,
    fmt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, ensure};
use object::{Object, ObjectSection, ObjectSymbol};
use serde::{Deserialize, Serialize};

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

    // Host binaries may keep defmt symbols in split sections. macOS uses
    // `.defmt,` section names because its linker restricts section names.
    let sections = elf
        .sections()
        .filter_map(|section| {
            section
                .name()
                .ok()
                .filter(|name| {
                    *name == ".defmt" || name.starts_with(".defmt.") || name.starts_with(".defmt,")
                })
                .map(|_| section.index())
        })
        .collect::<Vec<_>>();

    if sections.is_empty() {
        if version.is_none() {
            return Ok(None); // defmt is not used
        }
        bail!(
            "defmt version found, but no `.defmt` metadata section - check your linker configuration"
        );
    }
    let Some(version) = version else {
        bail!("found `.defmt` metadata sections, but no defmt version symbol");
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
    let mut collisions = BTreeSet::new();
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

        let Some(section_index) = entry.section_index() else {
            continue;
        };

        if sections.contains(&section_index) {
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

                    let section = elf.section_by_index(section_index)?;
                    let addr = entry.address();
                    let value = match section.data_range(addr, BITFLAGS_VALUE_SIZE)? {
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
                    let index = entry.address() as u16 as usize;
                    if collisions.contains(&index) {
                        log::warn!(
                            "defmt frame index collision at {index:#06x}; omitting symbol `{name}`"
                        );
                        continue;
                    }

                    let table_entry = TableEntry::new(
                        StringEntry::new(tag, sym.data().to_string()),
                        name.to_string(),
                    );
                    if let Some(old) = map.insert(index, table_entry) {
                        map.remove(&index);
                        collisions.insert(index);
                        // Keep a tombstone for ambiguous indices so later symbols cannot reuse them.
                        log::warn!(
                            "defmt frame index collision at {index:#06x}; omitting symbols `{}` and `{name}`",
                            old.raw_symbol
                        );
                    }
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
            "defmt wire format version mismatch: firmware is using {}, this tool supports {}\nsuggestion: install a newer version of this tool that supports defmt wire format version {}",
            version, DEFMT_VERSIONS.join(", "), version
        );

        return Err(msg);
    }

    Ok(())
}

/// Location of a defmt log statement in the elf-file
#[derive(Clone, Serialize, Deserialize)]
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

/// Mapping of decoded defmt frame index to [`Location`]
pub type Locations = BTreeMap<u64, Location>;

pub fn get_locations(elf: &[u8], table: &Table) -> Result<Locations, anyhow::Error> {
    let object = object::File::parse(elf)?;
    // Consumers look up locations by decoded frame index. Link locations by raw
    // symbol name so merged and split tables use the same table indices.
    let symbol_frame_indices = table
        .entries
        .iter()
        .map(|(index, entry)| (entry.raw_symbol.as_str(), *index as u64))
        .collect::<HashMap<_, _>>();
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
                // Do not use DW_AT_location here. Rust emits these as
                // `#[export_name] static DEFMT_LOG_STATEMENT`, and the symbol
                // table is the decoder's source of truth for the final u16
                // frame index. Raw linkage names let embedded merged tables,
                // host split tables, and biased host frames share the same
                // location mapping.

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
                        gimli::constants::DW_AT_linkage_name => {
                            if let gimli::AttributeValue::DebugStrRef(off) = attr.value() {
                                linkage_name = Some(off);
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(name_index), Some(linkage_name_index), Some(file_index), Some(line)) =
                    (name, linkage_name, decl_file, decl_line)
                {
                    let name_slice = dwarf.string(name_index)?;
                    let name = core::str::from_utf8(&name_slice)?;
                    let linkage_name_slice = dwarf.string(linkage_name_index)?;
                    let linkage_name = core::str::from_utf8(&linkage_name_slice)?;

                    if name == "DEFMT_LOG_STATEMENT" {
                        // DWARF may still mention defmt symbols that the linker
                        // garbage-collected. Link locations by raw symbol name.
                        if let Some(index) = symbol_frame_indices.get(linkage_name) {
                            let file = file_index_to_path(file_index, &unit, &dwarf)?;
                            let module = segments.join("::");

                            let loc = Location { file, line, module };

                            if let Some(old) = map.insert(*index, loc.clone()) {
                                bail!("BUG in DWARF variable filter: index collision for index 0x{:04x} (old = {:?}, new = {:?})", index, old, loc);
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

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use object::{
        write::{Object as WriteObject, Symbol, SymbolSection},
        Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
    };

    fn split_elf(symbols: impl IntoIterator<Item = (u64, String)>) -> Vec<u8> {
        split_elf_with_data(
            symbols
                .into_iter()
                .map(|(address, name)| (address, name, vec![0])),
        )
    }

    fn split_elf_with_data(symbols: impl IntoIterator<Item = (u64, String, Vec<u8>)>) -> Vec<u8> {
        elf_with_named_sections(
            symbols
                .into_iter()
                .enumerate()
                .map(|(index, (address, name, data))| {
                    (format!(".defmt.{index}"), address, name, data)
                }),
        )
    }

    fn elf_with_named_sections(
        symbols: impl IntoIterator<Item = (String, u64, String, Vec<u8>)>,
    ) -> Vec<u8> {
        let mut object =
            WriteObject::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
        object.add_file_symbol(b"split-defmt-test".to_vec());

        for (section_name, address, name, data) in symbols {
            let section = object.add_section(
                Vec::new(),
                section_name.into_bytes(),
                object::SectionKind::ReadOnlyData,
            );
            object.set_section_data(section, data.clone(), 1);
            object.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: address,
                size: data.len() as u64,
                kind: SymbolKind::Data,
                scope: SymbolScope::Linkage,
                weak: false,
                section: SymbolSection::Section(section),
                flags: SymbolFlags::None,
            });
        }

        for name in ["_defmt_version_ = 4", "_defmt_encoding_ = raw"] {
            object.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: 0,
                size: 0,
                kind: SymbolKind::Data,
                scope: SymbolScope::Compilation,
                weak: false,
                section: SymbolSection::Absolute,
                flags: SymbolFlags::None,
            });
        }

        object.write().unwrap()
    }

    fn log_symbol(data: &str, disambiguator: &str) -> String {
        format!(
            r#"{{"package":"pkg","tag":"defmt_info","data":"{data}","disambiguator":"{disambiguator}","crate_name":"crate"}}"#
        )
    }

    fn bitflags_value_symbol(data: &str) -> String {
        format!(
            r#"{{"package":"pkg","tag":"defmt_bitflags_value","data":"{data}","disambiguator":"a","crate_name":"crate"}}"#
        )
    }

    #[test]
    fn split_table_uses_symbol_address_index() {
        let elf = split_elf([(0x20, log_symbol("hello", "a"))]);
        let table = parse_impl(&elf, true).unwrap().unwrap();
        let mut frame = Vec::new();
        frame.write_u16::<LittleEndian>(0x20).unwrap();

        let (frame, consumed) = table.decode(&frame).unwrap();
        assert_eq!(consumed, 2);
        assert_eq!(frame.index(), 0x20);
        assert_eq!(frame.display_message().to_string(), "hello");
    }

    #[test]
    fn defmt_metadata_sections_are_gathered_by_name() {
        let elf = elf_with_named_sections([
            (".defmt".to_string(), 0, log_symbol("merged", "a"), vec![0]),
            (".defmt.1".to_string(), 1, log_symbol("split", "b"), vec![0]),
            (
                ".defmt,macos".to_string(),
                2,
                log_symbol("macos", "c"),
                vec![0],
            ),
        ]);
        let table = parse_impl(&elf, true).unwrap().unwrap();

        for (index, message) in [(0, "merged"), (1, "split"), (2, "macos")] {
            let mut frame = Vec::new();
            frame.write_u16::<LittleEndian>(index).unwrap();
            assert_eq!(
                table
                    .decode(&frame)
                    .unwrap()
                    .0
                    .display_message()
                    .to_string(),
                message
            );
        }
    }

    #[test]
    fn split_table_omits_symbols_after_prior_collision() {
        let elf = split_elf([
            (0, log_symbol("first", "a")),
            (0x1_0000, log_symbol("second", "b")),
            (0x2_0000, log_symbol("third", "c")),
        ]);
        let table = parse_impl(&elf, true).unwrap().unwrap();
        let mut frame = Vec::new();
        frame.write_u16::<LittleEndian>(0).unwrap();

        assert!(table.indices().next().is_none());
        assert_eq!(table.decode(&frame), Err(crate::DecodeError::Malformed));
        assert_eq!(
            table.decode_with_bias(&[3, 0], 3),
            Err(crate::DecodeError::Malformed)
        );
    }

    #[test]
    fn split_table_reads_bitflags_value_data_from_symbol_section() {
        let value = 0x1234u128;
        let elf = split_elf_with_data([(
            0,
            bitflags_value_symbol("Flags::0::A"),
            value.to_le_bytes().to_vec(),
        )]);
        let table = parse_impl(&elf, true).unwrap().unwrap();
        let values = table.bitflags.values().next().unwrap();

        assert_eq!(values, &[("A".to_string(), value)]);
    }
}
