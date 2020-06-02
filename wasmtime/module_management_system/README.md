# Wasmtime based module management system

- Install rust: https://www.rust-lang.org/tools/install
- Build the chatbot (refer to the chatbot `README.md` on how to build it)
- Build the module management system executable using `cargo build --release`.

After building the module management system (it will be located in the
`target/release/` folder), copy the generated `wasmtime` executable from the
`target/release` folder to the chatbot directory where all of the `wasm` files
are located `chatbot/target/wasm32-wasi/release`. Afterwards, you should be able
to execute the executable `./wasmtime` and use the chatbot.

