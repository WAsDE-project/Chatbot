use std::io;
use std::ffi::CString;
use std::os::raw::c_char;

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn Call(module_name: *const c_char, function_name: *const c_char) -> i32;
    pub fn Unload(module_name: *const c_char) -> i32;
    pub fn Load(module_name: *const c_char) -> i32;
}

struct Action<'a> {
    name: &'a str,
    func: fn() -> bool, // returns whether program should exit
}

fn print_options(actions: &[Action]) {
    for (index, action) in actions.iter().enumerate() {
        println!("{}) {}", index, action.name);
    }
}

fn print_main_message(actions: &[Action]) {
    println!("Select an action by typing its number:");
    print_options(&actions);
}

fn install_personality(personality: &str) {
    println!("Installing personality {}...", personality);
    let filename = CString::new(format!("{}.wasm", personality)).expect("Cant convert personality to c_str");
    let function_name = CString::new("run").expect("Cant convert function name to c_str");
    unsafe {
        if Load(filename.as_ptr()) != 0 {
            println!("Failed to load the personality {}!", personality);
            return;
        }
        if Call(filename.as_ptr(), function_name.as_ptr()) != 0 {
            println!(
                "An error occurred while running the personality {}!",
                personality
            );
        }
        if Unload(filename.as_ptr()) != 0 {
            println!(
                "An error occurred while unloading the personality {}!",
                personality
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let actions = [
        Action { name: "Exit", func: || true },
        Action { name: "Load personality: marvin", func: || { install_personality("marvin"); false } },
        Action { name: "Load personality: steve", func: || { install_personality("steve"); false } },
    ];

    print_main_message(&actions);
    loop {
        let mut personality = String::new();
        match io::stdin().read_line(&mut personality) {
            Ok(_size) => (),
            Err(e) => {
                println!("Unable to read a line: {:?}", e);
                continue;
            }
        }

        let index: u32 = match personality.trim().parse() {
            Ok(value) => value,
            Err(_) => {
                print_main_message(&actions);
                continue;
            }
        };

        let action = match actions.get(index as usize) {
            Some(value) => value,
            None => {
                println!("Choose one of the available options!");
                print_options(&actions);
                continue;
            }
        };

        if (&action.func)() {
            println!("Good bye!");
            break;
        }

        print_main_message(&actions);
    }
}
