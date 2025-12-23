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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Stopped,
    Running
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStopReason {
    Halted,
    CycleBudgetExhausted,
    Breakpoint(u16),
    Error,
}

pub struct Emulator {
    pub cpu: CPU,       // The meat
    pub bus: Bus,       // And potatoes

    // Related to the state of the machine, its cycles, and budget.
    run_state: RunState,
    cycles: u64,
    cycle_budget: Option<u64>,

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

            run_state: RunState::Stopped,
            cycles: 0,
            cycle_budget: None,

            rom: None,      
        }        
    }

    pub fn run_state(&mut self) -> RunState {
        self.run_state
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

        self.cpu.reset()?; // Registers and flags

        self.bus = Bus::new(Memory::new());

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

    // Control functions
    pub fn run(&mut self, cycles: Option<u64>) {
        self.cycle_budget = cycles;
        self.run_state = RunState::Running;
    }

    // Runs in a blocking fashion, until RunState tells it to stop
    pub fn run_blocking(&mut self, target_cycles: Option<u64>) -> RunStopReason {
        self.run(target_cycles);

        while self.run_state == RunState::Running {
            if self.cpu.is_halted() {
                self.stop();
                return RunStopReason::Halted;
            }

            let step = match self.cpu.step(&mut self.bus) {
                Ok(s) => s,
                Err(_) => {
                    self.stop();
                    return RunStopReason::Error;
                }
            };

            self.cycles += step.cycles as u64;

            if let Some(ref mut remaining) = self.cycle_budget {
                *remaining = remaining.saturating_sub(step.cycles as u64);
                if *remaining == 0 {
                    self.stop();
                    return RunStopReason::CycleBudgetExhausted;
                }
            }
        }

        RunStopReason::Error
    }
    
    pub fn stop(&mut self) {
        self.run_state = RunState::Stopped;
        self.cycle_budget = None;
    }

    pub fn step(&mut self) -> Option<StepResult> {
        if self.cpu.is_halted() {
            return None;
        }

        if let Ok(step) = self.cpu.step(&mut self.bus) {
            self.cycles += step.cycles as u64;
            return Some(step);
        }

        return None;
    }

}