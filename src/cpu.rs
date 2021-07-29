use std::fs::File;
use std::io::prelude::*;

// Memory related constants
pub const RAM_SIZE: usize = 0xFFFF;
pub const RAM_WORK_START: usize = 0x2000;
pub const RAM_VIDEO_START: usize = 0x2400;
pub const RAM_MIRROR_START: usize = 0x4000;

pub const OPCODE_SIZE: usize = 1;

enum ProgramCounter {
    Next,
    Two,
    Three,
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
    // A flag to indicate that we do not wish to execute, probably just printing disassembly
    pub nop: bool,
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
            nop: false,
        }
    }

    pub fn set_disassemble(&mut self, d: bool) {
        self.disassemble = d;
    }

    pub fn set_nop(&mut self, n: bool) {
        self.nop = n;
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

    // Gathers a word from memory based on program counter location,
    // then passes it along to the run_opcode() function
    pub fn tick(&mut self) {
        let opcode = self.read_opcode();
        self.run_opcode(opcode);
    }

    // Reads an instruction at ProgramCounter
    // Returns the following two bytes as potential "data" for the instruction.
    // If the two bytes are out of range they will return 0x00
    pub fn read_opcode(&mut self) -> (u8, u8, u8) {
        let o = match self.memory.get(self.pc) {
            Some(&v) => v,
            None => 0,
        };
        let x = match self.memory.get(self.pc + 1) {
            Some(&v) => v,
            None => 0,
        };
        let y = match self.memory.get(self.pc + 2) {
            Some(&v) => v,
            None => 0,
        };
        (o, x, y)
    }

    // This will parse the opcode, printing a disassembly if asked
    //
    // An opcode consists of:
    //  Instruction (1 byte)
    //  Data (1 or 2 bytes) depending on opcode.  Little endian.
    //
    pub fn run_opcode(&mut self, opcode: (u8, u8, u8)) {
        let x = opcode.1; // Potential data points for usage by an instruction
        let y = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0x00 => self.op_00(),     // NOP
            0xC3 => self.op_c3(x, y), // JMP
            0xC5 => self.op_c5(),     // PUSH B
            0xD5 => self.op_d5(),     // PUSH D
            0xE5 => self.op_e5(),     // PUSH H
            0xF5 => self.op_f5(),     // PUSH PSW
            _ => self.op_unk(),       // UNK
        };

        if self.disassemble {
            match i.size {
                ProgramCounter::Next => {
                    println!("{:#06X}\t{:#06X}\t\t\tCode: {}", self.pc, opcode.0, i.code)
                }
                ProgramCounter::Two => {
                    println!(
                        "{:#06X}\t{:#06X}\t{:#04X}\t\tCode: {}",
                        self.pc, opcode.0, x, i.code
                    )
                }
                ProgramCounter::Three => {
                    println!(
                        "{:#06X}\t{:#06X}\t{:#04X},{:#04X}\tCode: {}",
                        self.pc, opcode.0, x, y, i.code
                    )
                }
                _ => println!("TBD"),
            }
        }

        match i.size {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Two => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::Three => self.pc += OPCODE_SIZE * 3,
            ProgramCounter::Jump(d) => self.pc = d,
        }
    }

    pub fn op_00(&self) -> Instr {
        Instr {
            code: "NOP".to_string(),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_c3(&self, x: u8, y: u8) -> Instr {
        Instr {
            code: format!("JMP ${:02X}{:02X}", y, x),
            size: ProgramCounter::Three,
        }
    }

    pub fn op_c5(&self) -> Instr {
        Instr {
            code: format!("PUSH B"),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_d5(&self) -> Instr {
        Instr {
            code: format!("PUSH D"),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_e5(&self) -> Instr {
        Instr {
            code: format!("PUSH H"),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_f5(&self) -> Instr {
        Instr {
            code: format!("PUSH PSW"),
            size: ProgramCounter::Next,
        }
    }

    pub fn op_unk(&self) -> Instr {
        Instr {
            code: "!UNK!".to_string(),
            size: ProgramCounter::Next,
        }
    }
}
