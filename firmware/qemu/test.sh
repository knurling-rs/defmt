#!/usr/bin/env bash

set -e

FEATURES=${FEATURES-}

for testfile in src/bin/*.rs; do
    testname=$(basename "$testfile" .rs)

    cargo rb "$testname" --features "$FEATURES" | tee "src/bin/$testname.out.new" | diff "src/bin/$testname.out" -
    cargo rrb "$testname" --features "$FEATURES" | tee "src/bin/$testname.release.out.new" | diff "src/bin/$testname.release.out" -
done
