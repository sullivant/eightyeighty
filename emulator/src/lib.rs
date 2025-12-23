#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
pub mod cpu;
pub mod bus;
mod memory;
mod video;

use cpu::CPU;
use cpu::StepResult;

use crate::bus::Bus;
use crate::memory::Memory;

pub struct Emulator {
    pub cpu: CPU,
    pub bus: Bus,
    rom: Option<Vec<u8>>, // Storing the initial untouched rom, used when loading from new, or resetting.
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    /// Creates an empty, "powered off" machine.
    #[must_use] 
    pub fn new() -> Self {
        Emulator { 
            cpu: CPU::new(),
            bus: Bus::new(Memory::new()),
            rom: None,      
        }        
    }

    /// Inserts (readies) a rom into the machine.  But does not write anything to memory or reset the CPU.
    pub fn insert_rom(&mut self, rom: Vec<u8>) {
        self.rom = Some(rom);
    }

    /// Removes the ROM from the machine.
    pub fn remove_rom(&mut self) {
        self.rom = None;
    }

    /// Returns contents of ROM
    #[must_use] 
    pub fn rom(&self) -> Option<&[u8]> {
        self.rom.as_deref()
    }

    /// Resets ("reboots") the emulator and loads the ROM into memory
    /// 
    /// # Errors
    /// 
    /// Will return `Err` if we are not able to successfully insert a ROM.
    pub fn reset(&mut self) -> Result<(), String> {
        let rom = self.rom.as_ref().ok_or("No ROM Inserted")?;

        self.cpu.reset()?; // Registers, memory, etc.

        for (i, &b) in rom.iter().enumerate() {
            self.bus.write(i, b);
        }

        Ok(())
    }
 
    /// Inserts a rom and then ensures it loads into the CPU properly.  A convenience fn for "`insert_rom()`; `reset()`"
    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), String> {
        self.insert_rom(rom);
        self.reset()
    }

    /// Steps the CPU forward
    pub fn step(&mut self) -> Result<StepResult, String> {
        let result: StepResult = self.cpu.step(&mut self.bus)?;

        Ok(result)
    }


}