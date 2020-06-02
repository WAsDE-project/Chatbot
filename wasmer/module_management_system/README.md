# Wasmer based module management system
This folder implements a runtime with exposed custom Load, Unload, Call and CallWithMemory functions.

## Compiling
- Install rust: https://www.rust-lang.org/tools/install
- Build with `cargo build --release`
- The built binary is found at `target/release/wasmer_runtime`

## Running
The binary tries to run a `main.wasm` from the same folder.
You can execute the binary for example with `./wasmer_runtime`.
