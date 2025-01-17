#!/usr/bin/env bash

cd fuzz
cargo clean
rm Cargo.lock
cargo afl build
cargo afl fuzz -i in -o out target/debug/fuzz