#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
pub mod cpu;
mod memory;
mod video;

use cpu::CPU;
use cpu::StepResult;

pub struct Emulator {
    pub cpu: CPU,
    rom: Option<Vec<u8>>, // Storing the initial untouched rom, used when loading from new, or resetting.
}


impl Emulator {
    
    /// Creates an empty, "powered off" machine.
    pub fn new() -> Self {
        Emulator { 
            cpu: CPU::new(),
            rom: None,      
        }        
    }

    /// Inserts (readies) a rom into the machine.  But does not write anything to memory or reset the CPU.
    pub fn insert_rom(&mut self, rom: Vec<u8>) {
        self.rom = Some(rom);
    }

    /// Resets ("reboots") the emulator and loads the ROM into memory
    pub fn reset(&mut self) -> Result<(), String> {
        let rom = self.rom.as_ref().ok_or("No ROM Inserted")?;

        self.cpu.reset()?; // Registers, memory, etc.

        for (i, &b) in rom.iter().enumerate() {
            self.cpu.memory.write(i, b)?;
        }

        Ok(())
    }
 
    /// Inserts a rom and then ensures it loads into the CPU properly.  A convenience fn for "insert_rom(); reset()"
    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), String> {
        self.insert_rom(rom);
        self.reset()
    }

    /// Steps the CPU forward
    pub fn step(&mut self) -> Result<StepResult, String> {
        let result: StepResult = self.cpu.step()?;

        Ok(result)
    }


}