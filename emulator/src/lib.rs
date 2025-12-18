#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod constants;
pub mod cpu;
mod memory;
mod video;

use cpu::CPU;
use cpu::StepResult;

pub struct Emulator {
    pub cpu: CPU
}


impl Emulator {
    
    /// Creates an empty
    pub fn new() -> Self {
        Emulator { 
            cpu: CPU::new()
        }        
    }

    /// Loads a set of bytes that represent a rom, into memory
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        self.cpu.reset()?;

        for (i, b) in rom.iter().enumerate() {
            self.cpu.memory.write(i, *b)?;  // Write to memory, bubble up any troubles
        }

        Ok(())
    }


    /// Steps the CPU forward
    pub fn step(&mut self) -> Result<StepResult, String> {
        let result: StepResult = self.cpu.step()?;

        Ok(result)
    }


}