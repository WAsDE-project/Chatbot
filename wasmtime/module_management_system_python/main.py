import itertools

from wasmtime import Module, Store, Instance, WasiConfig, WasiInstance, FuncType, ValType, Func

# Global dictionary for instances.
INSTANCES = {}


# A helper function for gathering wasi imports.
def return_wasi_imports(store, module):
    wasi = WasiConfig()
    wasi.inherit_stdout()
    wasi.inherit_stdin()
    wasi.inherit_stderr()

    wasi = WasiInstance(store, "wasi_snapshot_preview1", wasi)

    return [wasi.bind(i) for i in module.imports() if i.module() == "wasi_snapshot_preview1"]


def load(caller, name):
    memory = caller.get_export("memory").memory()

    if memory is not None:
        store = Store()

        data = memory.data_ptr()[name:name + 15]
        module_name = "".join(map(chr, data[:data.index(0)]))
        module = Module.from_file(store, module_name)
        wasi_imports = return_wasi_imports(store, module)
        instance = Instance(module, wasi_imports)
        INSTANCES[module_name] = instance

    return 0


def call(caller, mod_name, func_name):
    memory = caller.get_export("memory").memory()
    if memory is not None:
        data = memory.data_ptr()[mod_name:mod_name + 15]
        module_name = "".join(map(chr, data[:data.index(0)]))
        data = memory.data_ptr()[func_name:func_name + 15]
        function_name = "".join(map(chr, data[:data.index(0)]))

        if module_name in INSTANCES:
            run = INSTANCES[module_name].get_export(function_name)
            run()

    return 0


def unload(caller, name):
    memory = caller.get_export("memory").memory()

    if memory is not None:
        data = memory.data_ptr()[name:name + memory.size()]
        module_name = "".join(map(chr, data[:data.index(0)]))

        del INSTANCES[module_name]

    return 0


class Runtime:
    def __init__(self):
        self.store = Store()
        self.module = Module.from_file(self.store, "main.wasm")
        self.imports = return_wasi_imports(self.store, self.module)

        load_callback_type = FuncType([ValType.i32()], [ValType.i32()])
        call_callback_type = FuncType([ValType.i32(), ValType.i32()], [ValType.i32()])
        unload_callback_type = FuncType([ValType.i32()], [ValType.i32()])

        load_callback_func = Func(self.store, load_callback_type, load, access_caller=True)
        call_callback_func = Func(self.store, call_callback_type, call, access_caller=True)
        unload_callback_func = Func(self.store, unload_callback_type, unload, access_caller=True)

        self.instance = Instance(self.module,
                                 [load_callback_func, call_callback_func, unload_callback_func, *self.imports])

    def return_instance(self):
        return self.instance


if __name__ == "__main__":
    runtime = Runtime()
    instance = runtime.return_instance()
    main = instance.get_export("main")

    main()
