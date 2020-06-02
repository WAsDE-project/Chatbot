#[macro_use]
extern crate lazy_static;

use std::{collections::HashMap, fs::File, io::Read, sync::Mutex};

use wasmer_runtime::{compile, func, Array, Ctx, Func, ImportObject, Instance, Module, WasmPtr};
use wasmer_runtime_core::import::Namespace;

use wasmer_wasi::{generate_import_object_from_state, get_wasi_version, state::WasiState};

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<String, Instance>> = { Mutex::new(HashMap::new()) };
}

fn main() {
    // Load and compile the main module.
    let mut file = File::open("main.wasm").unwrap();
    let mut module_bytes = Vec::new();
    file.read_to_end(&mut module_bytes).unwrap();
    let main_module = compile(&module_bytes).unwrap();
    // Create the ImportObject with WASI and our interface imported.
    let import_object = create_import_object("main.wasm", &main_module);
    // Instantiate the main module.
    let main_instance = main_module.instantiate(&import_object).unwrap();
    // Run the main module
    main_instance.call("main", &[]).unwrap();
}

fn create_import_object(name: &str, module: &Module) -> ImportObject {
    // Generate the import object with wasi-imports.
    let wasi_state = WasiState::new(name).build().unwrap();
    let wasi_version = get_wasi_version(module, false).unwrap();
    let mut import_object = generate_import_object_from_state(wasi_state, wasi_version);
    // Add our api imports.
    let mut ns = Namespace::new();
    ns.insert("Call", func!(call));
    ns.insert("CallWithMemory", func!(call_with_memory));
    ns.insert("Load", func!(load));
    ns.insert("Unload", func!(unload));
    import_object.register("env", ns);

    import_object
}

fn call(ctx: &mut Ctx, module_name: WasmPtr<u8, Array>, function_name: WasmPtr<u8, Array>) -> i32 {
    let caller_memory = ctx.memory(0);
    let module_name = module_name.get_utf8_string_with_nul(caller_memory).unwrap();

    // get the instance from the hashmap
    let instance: Instance = {
        let mut instance_map = INSTANCES.try_lock().unwrap();
        (*instance_map)
            .remove(module_name)
            .expect("module {} is either not loaded or is in use")
    };

    // get the function name from the caller's memory and call the function.
    let function_name = function_name
        .get_utf8_string_with_nul(caller_memory)
        .unwrap();
    let function: Func<(), ()> = instance.func(function_name).unwrap();
    let result = function.call();
    0
}

fn call_with_memory(
    ctx: &mut Ctx,
    module_name: WasmPtr<u8, Array>,
    function_name: WasmPtr<u8, Array>,
    source: WasmPtr<u8, Array>,
    source_len: u32,
    target: WasmPtr<u8, Array>,
    _target_len: u32,
) -> i32 {
    // Read the module name from memory
    let caller_memory = ctx.memory(0);
    let module_name = module_name.get_utf8_string_with_nul(caller_memory).unwrap();

    // get the instance from the hashmap
    let instance: Instance = {
        let mut instance_map = INSTANCES.try_lock().unwrap();
        (*instance_map)
            .remove(module_name)
            .expect("module {} is either not loaded or is in use")
    };

    //Ready the caller's parameter buffer.
    let memory_reader = source.deref(caller_memory, 0, source_len).unwrap();

    //Ready the callee's parameter buffer.
    let callee_memory = instance.context().memory(0);
    let get_parameter_buffer_pointer: Func<(), WasmPtr<u8, Array>> =
        instance.func("get_parameter_buffer_pointer").unwrap();
    let parameter_buffer = get_parameter_buffer_pointer.call().unwrap();
    let memory_writer = parameter_buffer
        .deref(callee_memory, 0, source_len)
        .unwrap();

    // Copy parameters between instance memories.
    for i in 0..source_len as usize {
        memory_writer[i].set(memory_reader[i].get());
    }

    // get the function name from the caller's memory and call the function.
    let function_name = function_name
        .get_utf8_string_with_nul(caller_memory)
        .unwrap();
    let function: Func<u32, u32> = instance.func(function_name).unwrap();
    let str_len = function.call(source_len).unwrap();

    //ready the callee return buffer.
    let get_return_buffer_pointer: Func<(), WasmPtr<u8, Array>> =
    instance.func("get_return_buffer_pointer").unwrap();
    let return_buffer = get_return_buffer_pointer.call().unwrap();
    let memory_reader = return_buffer.deref(callee_memory, 0, str_len).unwrap();

    //ready the caller return buffer
    let memory_writer = target.deref(caller_memory, 0, str_len).unwrap();
    
    // copy return value back to the caller.
    for i in 0..str_len as usize {
        memory_writer[i].set(memory_reader[i].get());
    }
    0
}

fn load(ctx: &mut Ctx, name: WasmPtr<u8, Array>) -> i32 {
    // Get the name of the module being loaded from the instance memory
    let memory = ctx.memory(0);
    let name = name.get_utf8_string_with_nul(memory).unwrap();

    // Load the module file into memory and compile it.
    // Potential safety issues here because we're just blindly taking the name from the memory
    // and opening a file with it.
    let mut file = File::open(name).unwrap();
    let mut module_bytes = Vec::new();
    file.read_to_end(&mut module_bytes).unwrap();
    let module = compile(&module_bytes).unwrap();

    // Create the ImportObject with WASI and our interface imported and instantiate module with it.
    let import_object = create_import_object(name, &module);
    let instance = module.instantiate(&import_object).unwrap();

    // Insert the module into the storage.
    let mut instance_map = INSTANCES.lock().unwrap();
    (*instance_map).insert(name.to_string(), instance);
    0
}

fn unload(ctx: &mut Ctx, name: WasmPtr<u8, Array>) -> i32 {
    let memory = ctx.memory(0);
    let name = name.get_utf8_string_with_nul(memory).unwrap();
    let mut instance_map = INSTANCES.lock().unwrap();
    (*instance_map).remove(name);
    0
}
