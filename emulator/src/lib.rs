#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
mod cpu;
mod memory;
mod video;

use crate::cpu::CPU;

/**
 * This library is, at its heart, simply a wrapper to the calls necessary to do
 * stuff within main.rs and cpu.rs.
 */

#[derive(Clone)]
pub struct Emulator {
    cpu: CPU,
}

impl Emulator {

    pub fn new() -> Emulator {
        Emulator { cpu: CPU::new() }
    }

    pub fn cpu_set_disassemble(&mut self, flag: bool) {
        self.cpu.disassemble(flag);
    }

    pub fn cpu_get_disassemble(&self) -> bool {
        self.cpu.disassemble
    }

    /// Since we cannot directly open a local file in Web Assembly, we need to expect
    /// the ROM data to come from the JavaScript side, on the browser.  So let's have
    /// a function here that will simply allow for the mapping of an array of data
    /// into the ROM space in memory.
    pub fn cpu_memory_write(&mut self, location: usize, data: u8) -> Result<bool, String> {
        match self.cpu.memory().write(location, data) {
            Ok(_) => (),
            Err(_) => ()
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

    pub fn cpu_tick(&mut self) -> u8 {
        match self.cpu.tick() {
            Ok(v) => v,
            Err(_) => {
                0
            }
        }
    }    

    pub fn cpu_reset(&mut self) -> bool {
        match self.cpu.reset() {
            Ok(_) => true,
            Err(_) => {
                false
            }
        }
    }

    pub fn cpu_get_vram(&self) -> String {
        format!("{:?}",self.cpu.memory.get_vram())
    }

}








