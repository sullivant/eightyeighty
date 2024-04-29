#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console::{self};
use cpu::instructions::{self, Instruction};

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

/// This returns a slice of memory, based off of a starting
/// address and consists of an array that is formatted in address/value pairs like this: [[0,255], [1,128]]
#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_get_memory(start: usize) -> String {

    unsafe {
        format!("{:?}",EMULATOR.cpu.memory.get_slice(start))
    }

    // unsafe { EMULATOR.cpu.memory.to_string() }
}


/// Returns an array containing all of the current register values as well as PC.
#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn get_all_registers() -> String {
    let mut ret: [usize; 9] = [0; 9];

    unsafe {
        let regs = EMULATOR.cpu.get_all_registers();
        // (&self.pc, &self.sp, &self.a, &self.b, &self.c, &self.d, &self.e, &self.h, &self.l)
        ret[0] = *regs.0; // PC
        ret[1] = *regs.1 as usize; // SP
        ret[2] = *regs.2 as usize; // A
        ret[3] = *regs.3 as usize; // B
        ret[4] = *regs.4 as usize; // C
        ret[5] = *regs.5 as usize; // D
        ret[6] = *regs.6 as usize; // E
        ret[7] = *regs.7 as usize; // H
        ret[8] = *regs.8 as usize; // L
    
        format!("{:?}", ret)
    }
}

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_state() -> String {
    unsafe { 
        EMULATOR.cpu.to_string() 
    }
}


#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_instructions() -> JsValue {
    unsafe {
        let mut ret = Vec::with_capacity(2);
        ret.push(EMULATOR.cpu.current_instruction);
        ret.push(EMULATOR.cpu.next_instruction);
        serde_wasm_bindgen::to_value(&ret).unwrap()
    }
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
pub fn cpu_next_instr() -> String {
    unsafe { EMULATOR.cpu.next_instruction.to_string() }
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

#[wasm_bindgen]
#[no_mangle]
#[must_use]
pub fn cpu_reset() -> bool {
    unsafe {
        match EMULATOR.cpu.reset() {
            Ok(_) => true,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                false
            }
        }
    }
}
