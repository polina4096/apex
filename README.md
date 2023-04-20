# Apex [![Build](https://github.com/polina4096/apex/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/polina4096/apex/actions/workflows/ci.yml) [![GitHub](https://img.shields.io/github/license/polina4096/apex)](https://github.com/polina4096/apex/blob/master/LICENSE) [![dependency status](https://deps.rs/repo/github/polina4096/apex/status.svg)](https://deps.rs/repo/github/polina4096/apex)

Highly hackable & performant taiko client and beatmap editor supporting multiple platforms such as Linux (X11 & Wayland), macOS, Windows and even web using WebAssembly. [Try it in the browser here!](https://polina4096.github.io/apex/)

## Build Instructions
1. Install the Rust toolchain: https://rustup.rs
2. Clone the repository: `git clone git@github.com:polina4096/apex.git`
3. Navigate to the cloned repository's directory: `cd apex`
4. Compile the program: `cargo build --release --package apex-client`

### WASM support
Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), build with: `wasm-pack build --release`.

Host a local web server with: `npm run start`.

## License
Distributed under the MIT license.