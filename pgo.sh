#!/bin/bash

export RUSTFLAGS="-Ctarget-cpu=x86-64-v3 -Ctarget-feature=+fma -Cprofile-generate=/tmp/pgo-data"

cargo run --package rust-v --release --all-features --  -v  -p 100  -fu16 -t6 -o "" ./scenes/prism.ron
cargo run --package rust-v --release --all-features --  -v  -p 100  -fu16 -t6 -o "" ./scenes/cornell.ron
cargo run --package rust-v --release --all-features --  -v  -p 10         -t6 -o "" ./scenes/dragon_4.ron
cargo run --package rust-v --release --all-features --  -v  -p 10         -t6 -o "" ./scenes/prism.ron
cargo run --package rust-v --release --all-features --  -v  -p 10         -t6 -o "" ./scenes/cornell.ron

~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata merge -o /tmp/merged.profdata /tmp/pgo-data

export RUSTFLAGS="-Ctarget-cpu=x86-64-v3 -Ctarget-feature=+fma -Cprofile-use=/tmp/merged.profdata"

cargo build --package rust-v --release --features show-image
cp ./target/release/main ./rust_v\ \(x86-64-v3\)