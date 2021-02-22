#!/bin/bash

export RUSTFLAGS="-Ctarget-cpu=native -Cprofile-generate=/tmp/pgo-data"

cargo run --package rust-v --release --features live-window -- cornell -v -l -p 1000 -d32 -fu16 -iPath -w64 -h64 -o ""
cargo run --package rust-v --release --features live-window -- cornell -v -l -p 100 -iPath -w128 -h128 -o ""
cargo run --package rust-v --release --features live-window -- spheres -v -l -p 100 -iPath -w128 -h128 -o ""
cargo run --package rust-v --release --features live-window -- debug -v -l -p 100 -iPath -w128 -h128 -o ""

~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata merge -o /tmp/merged.profdata /tmp/pgo-data

export RUSTFLAGS="-Ctarget-cpu=native -Cprofile-use=/tmp/merged.profdata"

cargo build --package rust-v --release --features live-window