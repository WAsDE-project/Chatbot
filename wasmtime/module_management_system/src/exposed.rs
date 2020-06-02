use crate::instance;

use std::borrow::BorrowMut;
use wasmtime::*;
use wasmtime_wasi::{Wasi, WasiCtx};

unsafe fn access_immutable_memory(memory: &Memory, name: i32) -> Option<String> {
    let data: &[u8] = &memory.data_unchecked()[name as usize..];
    for (index, byte) in (&data).iter().enumerate() {
        if byte.to_owned() as char == '\0' {
            match std::str::from_utf8(&data[..index]) {
                Ok(s) => return Some(s.to_string()),
                Err(_) => return None,
            }
        }
    }

    None
}

pub fn return_imports(module: &Module, store: &Store, kind: &str) -> Vec<Extern> {
    let mut imports = Vec::new();
    let wasi = Wasi::new(&store, WasiCtx::new(std::env::args()).expect("Wasi Error"));

    for import in module.imports() {
        if import.module() == kind {
            if let Some(export) = wasi.get_export(import.name()) {
                imports.push(Extern::from(export.clone()));
                continue;
            }
        }
    }

    imports
}

fn get_caller_memory(instance: &Caller) -> Memory {
    match instance.get_export("memory") {
        Some(Extern::Memory(memory)) => memory.clone(),
        _ => panic!("Cannot find exported memory."),
    }
}

pub fn load(store: &Store) -> Func {
    return Func::wrap(&store, |caller: Caller<'_>, name: i32| -> i32 {
        let memory = get_caller_memory(&caller);
        let mod_name = unsafe { access_immutable_memory(&memory, name).unwrap() };
        let new_store = Store::default();
        let module = Module::from_file(&new_store, &mod_name).expect("Cannot load a module");
        let imports = return_imports(&module, &new_store, "wasi_snapshot_preview1");
        let instance = Instance::new(&module, &imports).expect("Cannot instantiate a module");

        crate::INSTANCES.with(|mut i| {
            let new_instance = instance::ModuleInstance::new(mod_name, instance);
            (*i.borrow_mut()).lock().unwrap().push(new_instance);
        });

        return 0;
    });
}

pub fn call(store: &Store) -> Func {
    return Func::wrap(
        &store,
        |caller: Caller<'_>, name: i32, function: i32| -> i32 {
            let memory = get_caller_memory(&caller);
            let mod_name = unsafe { access_immutable_memory(&memory, name).unwrap() };
            let func_name = unsafe { access_immutable_memory(&memory, function).unwrap() };
            let mut return_value = 0;

            crate::INSTANCES.with(|mut instance| {
                for i in (*instance.borrow_mut()).lock().unwrap().iter() {
                    if i.name == mod_name {
                        let local_instance = &i.instance;

                        let function = local_instance
                            .get_export(func_name.as_str())
                            .unwrap()
                            .func()
                            .expect(
                                (format!("`{}` is not an exported function", func_name)).as_str(),
                            );

                        if let Err(_) = function.call(&[]) {
                            println!("Failed to call external function.");
                            return_value = 1;
                        };

                        break;
                    }
                }
            });

            return return_value;
        },
    );
}

pub fn unload(store: &Store) -> Func {
    return Func::wrap(&store, |caller: Caller<'_>, name: i32| -> i32 {
        let memory = get_caller_memory(&caller);
        let mod_name = unsafe { access_immutable_memory(&memory, name).unwrap() };

        crate::INSTANCES.with(|mut i| {
            let mut vector = (*i.borrow_mut()).lock().unwrap();
            let index = vector.iter().position(|p| p.name == mod_name).unwrap();

            vector.remove(index);
        });

        return 0;
    });
}
