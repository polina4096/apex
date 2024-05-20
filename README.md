# Apex
Hackable & performant taiko client supporting multiple platforms such as Linux (X11 & Wayland), macOS, Windows and even web using WebAssembly.

## Build Instructions
1. Install the Rust toolchain: https://rustup.rs
2. Clone the repository: `git clone git@github.com:polina4096/apex.git`
3. Navigate to the cloned repository's directory: `cd apex`
4. Compile the program: `cargo build --release --package apex-client`

This should generally work, but the project is not ready for testing yet. Expect no help if something goes wrong.

### WASM support
Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), build with: `wasm-pack build --release`.

Navigate to `crates/apex-client/web` and host a local web server with: `pnpm run dev`. This will not work as WASM build is a bit broken and not ready yet :)

## License
Distributed under the MIT license.
