# Rust\-_V_
A Rust-written ray tracer for my B.Sc. thesis. \
The _V_ stands for a ray reflection, and therefore figuratively for ray tracing.

## *NOTICE*
This project is WIP and actively being worked on.
Some demo scenes are available, but nothing "fancy" as of now.

## TODO
- Fix spectral integrator.

## Building
### Features
#### `f64`
Uses `f64` types for higher precision. It might be noticeable for some visuals,
but increases the runtime duration due to increased cache pressure.

#### `show-image`
By passing `--live` as runtime argument, the rendering will open in a window,
showing you the progress.

The window allows you some commands like following:
- `Ctrl + s`: Save current rendering as 8-bit PNG (with GUI ;-)

### Cargo
On the first build, _Cargo_ will need to download some crates as dependencies, just sit tight and wait a while. \
Run: \
`$  cargo build --package rust-v --bin main`

For a release (optimized) version, append `--release`: \
`$  cargo build --package rust-v --bin main --release`

For a live-window enabled version, append `--features "show-image"`: \
`$  cargo build --package rust-v --bin main --features "show-image"`

The compiled binary should be in the folder `./target/(dev|release)/rust_v`

## Scene files
We have some example scene files inside the `./scenes/` folder.

Refer to the syntax [here](./SceneSyntax.md) to write your own!.
