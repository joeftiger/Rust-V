#!/bin/bash
#SBATCH --mail-user=j.oeftiger@protonmail.com
#SBATCH --mail-type=end,fail
#SBATCH --job-name="Prism $1"
#SBATCH --time=24:00:00
#SBATCH --mem-per-cpu=16M
#SBATCH --cpus-per-task=128
#SBATCH --partition=amd
#SBATCH --exclusive
##SBATCH --test-only

ARGS="-iSpectralPath -d32 -p100 --bounds $2,$3,$4,$5 ./scenes/prism.ron --output ./prism_$1.png"
#echo "$ARGS"

export RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=+avx,+avx2,+fma"
cargo build --package rust-v --bin main

eval "./target/release/main ${ARGS}"
