#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;
mod utils;

use crate::cpu::CPU;
use std::fs::File;
use std::io::Read;
use wasm_bindgen::prelude::*;
use web_sys::console::{self};

#[derive(Clone)]
pub struct Emulator {
    cpu: CPU,
}

impl Emulator {
    const fn new() -> Emulator {
        Emulator{ cpu: CPU::new() }
    }
}

// Our actual emulator is really just a wrapped call to CPU
static mut EMULATOR: Emulator = Emulator::new();

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn cpu_greet() {
    alert(format!("Hello from WASM...").as_str());
}

#[wasm_bindgen]
#[no_mangle]
pub fn cpu_set_disassemble(flag: bool) {
    console::log_2(&"Setting disassemble flag to:".into(), &flag.into());
    unsafe {
        EMULATOR.cpu.disassemble(flag);
    }
}

#[wasm_bindgen]
#[no_mangle]
pub fn cpu_get_disassemble() -> bool {
    unsafe {
        EMULATOR.cpu.disassemble
    }
}

/// Since we cannot directly open a local file in Web Assembly, we need to expect
/// the ROM data to come from the JavaScript side, on the browser.  So let's have
/// a function here that will simply allow for the mapping of an array of data
/// into the ROM space in memory.
#[wasm_bindgen]
#[no_mangle]
pub fn cpu_memory_write(location: usize, data: u8) -> Result<bool, JsValue> {
    unsafe {
        match EMULATOR.cpu.memory().write(location, data) {
            Ok(_) => (),
            Err(e) => {
                console::log_1(&JsValue::from(format!("{}", e)));
            }
        };
    }

    Ok(true)
}

#[wasm_bindgen]
#[no_mangle]
pub fn cpu_state() -> String {
    unsafe {
        EMULATOR.cpu.to_string()
    }
}


//     // Actual threaded bits
//     let cpu = Arc::new(Mutex::new(Emulator::new(&rom_file)?)); // A threadable version of our emulator
//     let cpu_clone = Arc::clone(&cpu); // Used to tickle various settings

//     // If we are in debug mode, set that now, before we start
//     if matches.is_present("pause") {
//         println!("Setting pause on tick mode; <s> to step; <F1> to toggle; <F2> to kill CPU;");
//         cpu_clone.lock().unwrap().cpu.ok_to_step = false; // Will ensure we wait before executing first opcode!
//         cpu_clone.lock().unwrap().cpu.single_step_mode = true;
//     }

//     // Create a thread that will be our running cpu
//     // It's just gonna tick like a boss, until it's told not to.
//     let cpu_thread_handle = thread::spawn(move || {
//         while cpu_alive.load(Ordering::Relaxed) {
//             match cpu_clone.lock().unwrap().update() {
//                 Ok(_) => (),
//                 Err(e) => {
//                     println!("Unable to tick: {e}");
//                     break;
//                 }
//             }

//             // TODO: Make some form of cycle based speed regulation
//         }

//         println!(
//             "Shutting down. Final CPU state:\n{}",
//             cpu_clone.lock().unwrap().cpu
//         );
//     });

//     cpu_thread_handle.join().unwrap();
//     // lib::go()?;
//     Ok(())
// }

