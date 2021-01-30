# Rust\-_V_
A Rust-written ray tracer for my B.Sc. thesis. \
The _V_ stands for a ray reflection, and therefore figuratively for ray tracing.

## *NOTICE*
This project is WIP and actively being worked on.

Some demo scenes are available, but nothing "fancy" as of now.

## TODO
 - **Spectral types / paths **
 - Scene file parsing
 - More tests

## Building
### Features
#### `live-window`
By passing `--live` as runtime argument, the rendering will open in a window , showing you the progress.

The window allows you some commands like following:
- `Ctrl + s`: Save current rendering as 8-bit PNG (with GUI ;-)

### Cargo
On the first build, _Cargo_ will need to download some crates as dependencies, just sit tight and wait a while. \
Run: \
`$  cargo build --package rust-v --bin main`

For a release (optimized) version, append `--release`: \
`$  cargo build --package rust-v --bin main --release`

For a live-window enabled version, append `--features "live-window"`: \
`$  cargo build --package rust-v --bin main --features "live-window"`

The compiled binary should be in the folder `./target/(dev|release)/rust_v`
