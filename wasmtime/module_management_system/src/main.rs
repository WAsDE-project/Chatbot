mod exposed;
mod instance;

use anyhow::*;
use std::sync::Mutex;
use wasmtime::{Instance, Module, Store};

thread_local!(static INSTANCES: Mutex<Vec<instance::ModuleInstance>> = Mutex::new(Vec::new()));

fn main() -> Result<()> {
    let store = Store::default();

    let Load = exposed::load(&store);
    let Call = exposed::call(&store);
    let Unload = exposed::unload(&store);

    let module = Module::from_file(&store, "main.wasm")?;
    let imports = exposed::return_imports(&module, &store, "wasi_snapshot_preview1");
    let import_functions = &[Load.into(), Call.into(), Unload.into()];
    let all_imports = [&import_functions[..], &imports[..]].concat();

    let instance = Instance::new(&module, &all_imports)?;

    let load_results = instance
        .get_export("main")
        .unwrap()
        .func()
        .expect("Cannot call the main function");

    if let Err(e) = load_results.call(&[]) {
        panic!(e);
    }

    Ok(())
}
