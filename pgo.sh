#!/bin/bash

export RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=+fma -Cprofile-generate=/tmp/pgo-data"

cargo run --package rust-v --release --all-features -- -v -p 100 -iSpectralPath -d32 -fu16 -iPath -o "" ./prism.ron
cargo run --package rust-v --release --all-features -- -v -p 100 -iPath -d32 -fu16 -iPath -o "" ./prism.ron
cargo run --package rust-v --release --all-features -- -v -p 10 -iSpectralPath -o "" ./prism.ron
cargo run --package rust-v --release --all-features -- -v -p 10 -iPath -o "" ./prism.ron
cargo run --package rust-v --release --all-features -- -v -p 10 -iSpectralPath4 -o "" ./prism.ron
cargo run --package rust-v --release --all-features -- -v -p 10 -iPath -o "" ./prism.ron

~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata merge -o /tmp/merged.profdata /tmp/pgo-data

export RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=+fma -Cprofile-use=/tmp/merged.profdata"

cargo build --package rust-v --release --all-features