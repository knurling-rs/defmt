# Timestamp

> [!IMPORTANT]
> The design and implementation chapter is outdated

`defmt::timestamp!` needs to be as efficient as possible, because it is implicitly invoked on every single log invocation.

The timestamp format string index is not transmitted over the wire.
Instead, it is marked with a `defmt_timestamp` tag and the decoder loads it from the ELF file.
Linker magic is used to make sure that it doesn't get defined twice, and that the symbol doesn't get discarded (which can happen since its address is never used).

The `us` format specifier was introduced to allow replicating the timestamp format of previous defmt versions, which always used a LEB128-encoded `u64` timestamp and treated it as a number of microseconds.
