# Lookup

> ⚠️ The design and implementation chapter is outdated ⚠️

We have so far looked at the string table using `nm`.
Programmatically the table can be found in the `.symtab` section.
Each [entry] in this table represents a symbol and each entry has:
- `shndx`, a section header index (?). This should match the index of the `.defmt` section.
- `value`, this is the address of the symbol. For `.defmt`, this is the string index
- `name`, an index into some data structure full of strings. `get_name` returns the interned string
- the other info is not relevant

[entry]: https://docs.rs/xmas-elf/0.7.0/xmas_elf/symbol_table/trait.Entry.html
