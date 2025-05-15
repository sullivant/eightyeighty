#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;
use constants::{VRAM_SIZE, VRAM_START};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console::{self};
use cpu::instructions::{self, Instruction};
use std::f64;

/**
 * This library is, at its heart, simply a WASM bound wrapper to the calls necessary to do
 * stuff within main.rs and cpu.rs.
 */

#[derive(Clone)]
#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
}

#[wasm_bindgen]
impl Emulator {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator { cpu: CPU::new() }
    }

    pub fn cpu_set_disassemble(&mut self, flag: bool) {
        console::log_2(&"Setting disassemble flag to:".into(), &flag.into());
        self.cpu.disassemble(flag);
    }

    pub fn cpu_get_disassemble(&self) -> bool {
        self.cpu.disassemble
    }

    /// Since we cannot directly open a local file in Web Assembly, we need to expect
    /// the ROM data to come from the JavaScript side, on the browser.  So let's have
    /// a function here that will simply allow for the mapping of an array of data
    /// into the ROM space in memory.
    pub fn cpu_memory_write(&mut self, location: usize, data: u8) -> Result<bool, JsValue> {
        match self.cpu.memory().write(location, data) {
            Ok(_) => (),
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
            }
        };

        Ok(true)
    }

    pub fn cpu_get_memory_ptr(&self) -> *const u8 {
        self.cpu.memory.get_memory_ptr()
    }

    pub fn cpu_get_memory_size(&self) -> usize {
        self.cpu.memory.get_memory_size()
    }

    /// This returns a 16 byte slice of memory, based off of a starting
    /// address and consists of an array that is formatted in address/value pairs like this: [[0,255], [1,128]]
    pub fn cpu_get_memory(&self, start: usize) -> String {
        format!("{:?}",self.cpu.memory.get_slice(start))
    }


    /// Returns an array containing all of the current register values as well as PC.
    pub fn cpu_registers(&self) -> String {
        let mut ret: [usize; 9] = [0; 9];

        let regs = self.cpu.get_all_registers();
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

    pub fn cpu_state(&self) -> String {
        self.cpu.to_string() 
    }

    pub fn cpu_instructions(&self) -> JsValue {
        let mut ret: Vec<_> = Vec::with_capacity(2);
        ret.push(self.cpu.current_instruction);
        ret.push(self.cpu.next_instruction);
        serde_wasm_bindgen::to_value(&ret).unwrap()
    }

    pub fn cpu_tick(&mut self) -> u8 {
        match self.cpu.tick() {
            Ok(v) => v,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                0
            }
        }
    }    

    pub fn cpu_reset(&mut self) -> bool {
        match self.cpu.reset() {
            Ok(_) => true,
            Err(e) => {
                console::log_1(&JsValue::from(e.to_string()));
                false
            }
        }
    }

    pub fn cpu_get_vram(&self) -> String {
        format!("{:?}",self.cpu.memory.get_vram())
    }

}

// Our actual emulator is really just a wrapped call to CPU
// static mut EMULATOR: Emulator = Emulator::new();

#[wasm_bindgen]
#[must_use]
extern "C" {
    fn alert(s: String);

    // Various forms of logging with different signatures
    // Thank you  wasm-bindgen guide.  Of course we can also use
    // console::log_2(...)
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}






