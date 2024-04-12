#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;
use wasm_bindgen::prelude::*;
use web_sys::console::{self};

/**
 * This library is, at its heart, simply a WASM bound wrapper to the calls necessary to do
 * stuff within main.rs and cpu.rs.
 */

#[derive(Clone)]
pub struct Emulator {
    cpu: CPU,
}

impl Emulator {
    const fn new() -> Emulator {
        Emulator { cpu: CPU::new() }
    }
}

// Our actual emulator is really just a wrapped call to CPU
static mut EMULATOR: Emulator = Emulator::new();

#[wasm_bindgen]
#[must_use]
extern "C" {
    fn alert(s: String);
}

#[wasm_bindgen]
pub fn cpu_greet() {
    alert("Hello from WASM!".to_string());
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
#[must_use]
pub fn cpu_get_disassemble() -> bool {
    unsafe { EMULATOR.cpu.disassemble }
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
                console::log_1(&JsValue::from(e.to_string()));
            }
        };
    }

    Ok(true)
}

/// This returns all the memory.  Which... whatever.  Could probably be paged
/// in such a way as to only return what's requested.  "(operating)Memory is cheap?"
#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_get_memory() -> String {
    unsafe { EMULATOR.cpu.memory.to_string() }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_state() -> String {
    unsafe { EMULATOR.cpu.to_string() }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_curr_instr() -> String {
    unsafe { EMULATOR.cpu.current_instruction.to_string() }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_tick() -> bool {
    unsafe {
        match EMULATOR.cpu.tick() {
            Ok(_) => true,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                false
            }
        }
    }
}
