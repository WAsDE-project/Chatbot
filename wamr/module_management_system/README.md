# WAMR based module management system
The example folder has code to create a runtime with exposed custom Load, Unload, Call and CallWithMemory functions.

## Building
- Clone wamr `git clone https://github.com/bytecodealliance/wasm-micro-runtime.git`
- Go inside the cloned folder `cd wasm-micro-runtime`
- Use `git checkout d381b0fdec04f5ca8b891c070a00fd99a785c322` to get the wamr version this example was built on
- Put the `example` folder inside `product-mini/platforms` folder
- Go inside the `example` folder that you just moved and build the example with `mkdir build && cd build && cmake .. && make`
- The built `iwasm` binary is inside the `build` folder.

## Running
When the binary is run it will search for a `main.wasm` file in the same folder and attempts to execute it as a WASM module by executing its main function.

You can supply a different file name to execute by giving it as an argument to the binary when running it.

The binary has two different memory allocation strategies. One uses malloc and free and is used by default. The other one uses a preallocated buffer and does not work properly with dynamically loaded modules. You can select the buffer allocation strategy by supplying a second argument to the runtime when running a wasm module.

Examples of running the runtime:
- `./iwasm`
- `./iwasm module.wasm`
- `./iwasm module.wasm 1`
