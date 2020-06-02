
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <stdlib.h>
#include <string.h>
#include "bh_platform.h"
#include "bh_assert.h"
#include "bh_log.h"
#include "bh_read_file.h"
#include "wasm_export.h"

// global definitions of wasm runtime stack and heap sizes in bytes
const uint32 stack_size = 100 * 1024 * 1024, heap_size = 100 * 1024 * 1024;

// Struct holding all module's data
// also holds a pointer to next module in the global linked list
typedef struct moduledata {
    char* name;
    uint8_t* wasm; // code
    uint32_t wasm_size; // code length
    char error_buf[128];
    wasm_module_t module;
    wasm_module_inst_t module_inst;
    wasm_exec_env_t exec_env;
    struct moduledata* next; // Next item in linked list
} ModuleData;

/*
    Frees the module and returns the next module in the list.
    
    @param data: the module to free
    @return the next module in the linked list or NULL
*/
ModuleData* FreeModule(ModuleData* data) {
    if (!data)
        return NULL;
    ModuleData* next = data->next;
    if (data->exec_env)
        wasm_runtime_destroy_exec_env(data->exec_env);
    if (data->module_inst)
        wasm_runtime_deinstantiate(data->module_inst);
    if (data->module)
        wasm_runtime_unload(data->module);
    if (data->wasm)
        wasm_runtime_free(data->wasm);
    if (data->name)
        free(data->name);
    free(data);
    return next;
}

// A global linked list of all loaded modules
ModuleData* Modules = NULL;

/*
    This function can be called by wasm modules to load other modules

    The function loads a module by the given name if found.
    Then it stores it in a list.
    
    @param wasmname: the name of the wasm module to load
    @return 0 in case of success and 1 in case of a failure
*/
ModuleData* LoadModule(const char* wasmname) {

    // Allocate the structure holding the module's data
    ModuleData* data = malloc(sizeof (ModuleData));
    if (!data) {
        return NULL;
    }
    data->name = NULL;
    data->wasm = NULL;
    data->wasm_size = 0;
    data->module = NULL;
    data->module_inst = NULL;
    data->exec_env = NULL;
    data->next = NULL;

    // Allocate space for the name of the module
    if (!(data->name = malloc((strlen(wasmname)+1) * sizeof(char)))) {
        printf("Cannot allocate memory for module name\n");
        FreeModule(data);
        return NULL;
    }
    strcpy(data->name, wasmname);

    // Read the wasm module from file
    data->wasm = (uint8_t*)bh_read_file_to_buffer(wasmname, &data->wasm_size);
    // const char *err = getfile(wasmname, &data->wasm, &data->wasm_size);
    if (!data->wasm)
    {
        printf("Failed to load file\n");
        FreeModule(data);
        return NULL;
    }
    
    // Load and instantiate a module
    if (!(data->module = wasm_runtime_load(data->wasm, data->wasm_size, data->error_buf, sizeof(data->error_buf)))) {
        printf("%s\n", data->error_buf);
        FreeModule(data);
        return NULL;
    }

#if WASM_ENABLE_LIBC_WASI != 0
    const char *dir_list[8] = { NULL };
    uint32 dir_list_size = 0;
    const char *env_list[8] = { NULL };
    uint32 env_list_size = 0;
    wasm_runtime_set_wasi_args(data->module,
                               dir_list, dir_list_size,
                               NULL, 0,
                               env_list, env_list_size,
                               NULL, 0);
#endif

    if (!(data->module_inst = wasm_runtime_instantiate(data->module, stack_size, heap_size, data->error_buf, sizeof(data->error_buf)))) {
        printf("%s\n", data->error_buf);
        FreeModule(data);
        return NULL;
    }
    if (!(data->exec_env = wasm_runtime_create_exec_env(data->module_inst, stack_size))) {
        printf("Could not create exec_env!\n");
        FreeModule(data);
        return NULL;
    }

    // Apparently _start must be called if it exists
    // before calling any other functions
    wasm_function_inst_t func = wasm_runtime_lookup_function(data->module_inst, "_start", "()");
    if (func)
    {
        uint32 argv[] = {0,0}; // 64 bit uses 2 values for return
        if (!wasm_runtime_call_wasm(data->exec_env, func, 0, argv))
        {
            printf("%s\n", wasm_runtime_get_exception(data->module_inst));
            FreeModule(data);
            return NULL;
        }
    }

    // We dont need the code anymore after loading it
    free(data->wasm);
    data->wasm = NULL;
    data->wasm_size = 0;

    return data;
}

/*
    This function can be called by wasm modules to load other modules

    The function loads a module by the given name if found.
    Then it stores it in a list.
    
    @param exec_env: the runtime's own execution environment
    @param wasmname: the name of the wasm module to load
    @return 0 in case of success and 1 in case of a failure
*/
int32 Load(wasm_exec_env_t exec_env, const char* wasmname)
{
    // Load a module
    ModuleData* data = LoadModule(wasmname);
    if (!data) {
        printf("Could not load module %s!\n", wasmname);
        return 1;
    }

    // store it in a linked list
    data->next = Modules;
    Modules = data;
    return 0;
}

/*
    This function can be called by wasm modules to unload a loaded module

    The function unloads a module by the given name if found.
    It will free the module and all of it's reserved memory.
    
    @param exec_env: the runtime's own execution environment
    @param wasmname: the name of the wasm module to unload
    @return 0 in case of success and 1 in case of a failure
*/
int32 Unload(wasm_exec_env_t exec_env, char* wasmname)
{
    // find module by name
    ModuleData* prev = NULL;
    ModuleData* it = Modules;
    while (it) {
        if (!strcmp(it->name, wasmname)) {
            break;
        }
        prev = it;
        it = it->next;
    }
    if (!it) {
        printf("No module %s loaded!\n", wasmname);
        return 1;
    }

    // Free the module and remove from linked list
    if (prev)
        prev->next = FreeModule(it);
    else {
        FreeModule(it);
        Modules = NULL;
    }
    return 0;
}

/*
    This function can be called by the wasm modules to call functions from other modules.

    The function searches a loaded module with the given name. Then it will find a function with the given name.
    The found function is then called.
    In the end we return the return value of the called function.

    @param exec_env: the runtime's own execution environment
    @param wasmname: the name of the wasm module to call the function from
    @param functionname: the name of the function to call
    @return 0 in case of success and 1 in case of a failure
*/
int32 Call(wasm_exec_env_t exec_env, char* wasmname, char* functionname)
{
    // find module by name
    ModuleData* data = Modules;
    while (data) {
        if (!strcmp(data->name, wasmname)) {
            break;
        }
        data = data->next;
    }
    if (!data) {
        printf("No module %s loaded!\n", wasmname);
        return 1;
    }
    
    // find the function by name
    wasm_function_inst_t func = wasm_runtime_lookup_function(data->module_inst, functionname, "()i32");
    if (!func)
    {
        printf("Could not find function %s from module %s!\n", functionname, wasmname);
        return 1;
    }

    // Call the function
    // The return value of the called function is placed in the parameter array
    uint32 argv[2] = {}; // 64 bit uses 2 values for return
    if (!wasm_runtime_call_wasm(data->exec_env, func, 0, argv))
    {
        printf("%s\n", wasm_runtime_get_exception(data->module_inst));
        return 1;
    }

    /* the return value is stored in argv[0] */
    int retval = argv[0];

    // return called function's return value
    return retval;
}

/*
    This function can be called by the wasm modules to call functions from other modules.

    The function searches a loaded module with the given name. Then it will find a function with the given name.
    Then it will allocate memory in the found module for buffers of given lengths and copy the data from
    the given buffers to the allocated ones. The found function is then called with these allocated buffers as arguments.
    Once the function call ends we validate that the return buffer is still valid. We then copy the return buffer's contents
    to the caller's memory buffer and free the previously allocated memory from the found module.
    In the end we return the return value of the called function.

    @param exec_env: the runtime's own execution environment
    @param wasmname: the name of the wasm module to call the function from
    @param functionname: the name of the function to call
    @param src_buf: Buffer containing data passed to the called function
    @param src_len: Length of the buffer
    @param src_buf: Buffer for the return value of the called function
    @param src_len: Length of the buffer
    @return 0 in case of success and 1 in case of a failure
*/
int32 CallWithMemory(wasm_exec_env_t exec_env, char* wasmname, char* functionname, char * src_buf, uint32_t src_len, char * dest_buf, uint32_t dest_len)
{
    // find module by name
    ModuleData* data = Modules;
    while (data) {
        if (!strcmp(data->name, wasmname)) {
            break;
        }
        data = data->next;
    }
    if (!data) {
        printf("No module %s loaded!\n", wasmname);
        return 1;
    }
    
    // find the function by name
    wasm_function_inst_t func = wasm_runtime_lookup_function(data->module_inst, functionname, "(i32i32i32i32)i32");
    if (!func)
    {
        printf("Could not find function %s from module %s!\n", functionname, wasmname);
        return 1;
    }

    // Allocate memory in the found module with the size of the caller's buffers
    // and copy the caller's buffer's contents to the allocated buffers 
    int32_t src_index = wasm_runtime_module_dup_data(data->module_inst, src_buf, src_len);
    if (!src_index) {
        printf("Could not allocate memory for function call src!\n");
        return 1;
    }
    int32_t dest_index = wasm_runtime_module_dup_data(data->module_inst, dest_buf, dest_len);
    if (!dest_index) {
        printf("Could not allocate memory for function call dest!\n");
        return 1;
    }

    // Pass the allocated memory buffer indexes to the called function and call the function
    // The return value of the called function is placed in the parameter array
    uint32 argv[] = {src_index, src_len, dest_index, dest_len}; // 64 bit uses 2 values for return
    if (!wasm_runtime_call_wasm(data->exec_env, func, 4, argv))
    {
        printf("%s\n", wasm_runtime_get_exception(data->module_inst));
        return 1;
    }

    /* the return value is stored in argv[0] */
    int retval = argv[0];

    // do boundary check for the buffer
    if (!wasm_runtime_validate_app_addr(data->module_inst, dest_index, dest_len))
        return 1;

    // do address conversion from wasm index to pointer for the buffer
    char* ret_buffer = wasm_runtime_addr_app_to_native(data->module_inst, dest_index);
    
    // Copy the data from return buffer to the return buffer of the caller
    strncpy(dest_buf, ret_buffer, dest_len);

    // Free previously allocated buffers from the found module
    wasm_runtime_module_free(data->module_inst, src_index);
    wasm_runtime_module_free(data->module_inst, dest_index);

    // return called function's return value
    return retval;
}


// An array of host functions to expose to wasm modules
// They can be imported and called by the wasm modules
// Read more about this here: https://github.com/bytecodealliance/wasm-micro-runtime/blob/master/doc/export_native_api.md
static NativeSymbol native_symbols[] =
{
    EXPORT_WASM_API_WITH_SIG(Call, "($$)i"),
    EXPORT_WASM_API_WITH_SIG(CallWithMemory, "($$*~*~)i"),
    EXPORT_WASM_API_WITH_SIG(Load, "($)i"),
    EXPORT_WASM_API_WITH_SIG(Unload, "($)i")
};

int main(int argc, char *argv[])
{
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(RuntimeInitArgs));

    // There are two different allocator settings we can use, but the other does not work properly with multiple runtimes
    // default to the working one unless an extra parameter passed to exe
    if (argc == 3) {
        // Doesnt work, seems to cause UB and Segmentation fault (core dumped)
        // when used with multiple modules at least
        static char global_heap_buf[10 * 1024 * 1024] = { 0 };
        init_args.mem_alloc_type = Alloc_With_Pool;
        init_args.mem_alloc_option.pool.heap_buf = global_heap_buf;
        init_args.mem_alloc_option.pool.heap_size = sizeof(global_heap_buf);
    } else {
        init_args.mem_alloc_type = Alloc_With_Allocator;
        init_args.mem_alloc_option.allocator.malloc_func = malloc;
        init_args.mem_alloc_option.allocator.realloc_func = realloc;
        init_args.mem_alloc_option.allocator.free_func = free;
    }

    // Set the exposed host functions
    init_args.n_native_symbols = sizeof(native_symbols) / sizeof(NativeSymbol);
    init_args.native_module_name = "env";
    init_args.native_symbols = native_symbols;

    // initialize runtime
    if (!wasm_runtime_full_init(&init_args)) {
        printf("Init runtime environment failed.\n");
        return 1;
    }
    bh_log_set_verbose_level(2);

    // Default to main.wasm, but can also pass in a file name as parameter
    char* main_name = "main.wasm";
    if (argc >= 2) main_name = argv[1];

    // Load the main module
    ModuleData* data = LoadModule(main_name);
    if (!data) {
        printf("Couldnt load %s!\n", main_name);
    } else {
        if (!wasm_application_execute_main(data->module_inst, 0, NULL))
        {
            printf("Runtime exception: %s\n", wasm_runtime_get_exception(data->module_inst));
            goto _onfatal;
        }
    }

_onfatal: ;

    // unload modules that were not unloaded during program execution
    ModuleData* it = Modules;
    while (it) {
        it = FreeModule(it);
    }

    // unload main module
    FreeModule(data);

    // free runtime
    wasm_runtime_destroy();
    return 0;
}
