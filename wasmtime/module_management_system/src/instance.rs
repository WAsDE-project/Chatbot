use wasmtime::Instance;

pub struct ModuleInstance {
    pub name: String,
    pub instance: Instance,
}

impl ModuleInstance {
    pub fn new(name: String, instance: Instance) -> ModuleInstance {
        ModuleInstance { name, instance }
    }
}
