#!/bin/bash

bins=("assert" "assert-eq" "assert-ne" "bitflags" "dbg" "hints" "hints_inner" "log" "panic" "panic_info" "timestamp" "unwrap")

echo "Generating output ..."

for value in "${bins[@]}"; do
	command="DEFMT_LOG=trace cargo -q run --features no-decode --manifest-path ../../qemu-run/Cargo.toml ../target/thumbv7m-none-eabi/debug/$value > ~/defmt/xtask/output_files/$value.out"
	echo "$command"
	eval "$command"
done
