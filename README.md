# Apex
Hackable & performant taiko client supporting multiple platforms such as Linux (X11 & Wayland), macOS, Windows and even web using WebAssembly.

## Build Instructions
1. Install the Rust toolchain: https://rustup.rs
2. Install cargo-make: `cargo install --force cargo-make`
3. Clone the repository: `git clone git@github.com:polina4096/apex.git`
4. Navigate to the cloned repository's directory: `cd apex`
5. Compile the program: `cargo make build`

This should generally work, but the project is not ready for testing yet. Expect no help if something goes wrong for now.

Actually, build instructions do not work at all. There is no support for release builds yet, and if you want to just run a debug one do: `cargo make`. You can do `cargo build --release --bin apex-client`, but it will only build the executable which is not guaranteed to run with improper directory structure.

### WASM support
Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), build with: `wasm-pack build --release`.

Navigate to `crates/apex-client/web` and host a local web server with: `pnpm run dev`. This will not work as WASM build is a bit broken and not ready yet :)

## License
Distributed under the MIT license.
