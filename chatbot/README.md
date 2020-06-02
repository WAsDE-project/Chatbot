# Chatbot
A commandline implementation of a chatbot that can dynamically load, unload and execute personalties implemented as separate WASM modules.

We have included precompiled modules and runtimes to the `precompiled` folder. The precompiled runtimes may not work on every system, they are built for 64 bit linux. In case they do not work for you follow the instructions below for building them.

## Compiling
- Install rust: https://www.rust-lang.org/tools/install
- Install cargo-wasi: `cargo install cargo-wasi`
- Build: `cargo wasi build --release`
- The built files `main.wasm`, `marvin.wasm` and `steve.wasm` are located in `target/wasm32-wasi/release` folder.
- You are going to need a runtime to run the WASM modules on. Build one of the available runtimes from `../wamr/module_management_system`, `../wasmer/module_management_system` or `../wasmtime/module_management_system` by following the instructions in their README files

## Running
- Move the WASM files and a runtime into the same folder
- Run `main.wasm` with a runtime. For example by using the command `./iwasm`, `./wasmtime` or `./wasmer_runtime` in the folder containing the modules and the runtime.
- When starting the program the program will show instructions on screen. You can type in numbers to select specific actions. Once you have selected to install a personality you can input text to which the personality then reacts. To exit a personality you can input the command `exit`.
