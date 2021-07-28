use std::fs::File;
use std::io::prelude::*;

// Memory related constants
pub const RAM_SIZE: usize = 0xFFFF;
pub const RAM_WORK_START: usize = 0x2000;
pub const RAM_VIDEO_START: usize = 0x2400;
pub const RAM_MIRROR_START: usize = 0x4000;

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

pub struct Instr {
    code: String,
    size: ProgramCounter,
}

pub struct Cpu {
    // Memory
    pub memory: [u8; RAM_SIZE],

    // Registers
    pub pc: usize, // Program Counter

    // A flag that indicates we wish to print human readable command references
    pub disassemble: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            memory: [0; RAM_SIZE],
            pc: 0x00,
            disassemble: false,
        }
    }

    pub fn set_disassemble(&mut self, d: bool) {
        self.disassemble = d;
    }

    // Load the ROM file into memory, starting at start_index
    // Returns a tuple containing the index we started at and where we
    // actually finished at.
    pub fn load_rom(
        &mut self,
        file: String,
        start_index: usize,
    ) -> Result<(usize, usize), std::io::Error> {
        let rom = File::open(file)?;
        let mut last_idx: usize = 0;
        for (i, b) in rom.bytes().enumerate() {
            self.memory[start_index + i] = b.unwrap();
            last_idx = i;
        }
        Ok((start_index, start_index + last_idx + 1))
    }

    // This will parse the opcode, printing a disassembly if asked
    //
    // TODO: I really think this can be worked into "run_opcode(...)"
    //
    // Parameters:
    //   byte: &u8
    pub fn parse_opcode(&self, opcode: &u8) {
        //TODO: This can be a slice of bytes, up to 3 depending on the current need
        let i = match opcode {
            0x00 => self.op_00(),  // NOP
            0xC3 => self.op_jmp(), // JMP
            _ => self.op_unk(),    // UNK
        };

        if self.disassemble {
            println!("\t{:#04X}\tCode:{}", opcode, i.code);
        }
    }

    pub fn op_00(&self) -> Instr {
        Instr {
            code: "NOP".to_string(),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_jmp(&self) -> Instr {
        Instr {
            code: "JMP".to_string(),
            size: ProgramCounter::Skip,
        }
    }

    pub fn op_unk(&self) -> Instr {
        Instr {
            code: "!UNK!".to_string(),
            size: ProgramCounter::Next,
        }
    }
}
