use std::fs::File;
use std::io::prelude::*;

pub struct Cpu {
    // Hello there
    pub disassemble: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { disassemble: false }
    }

    pub fn set_disassemble(&mut self, d: bool) {
        self.disassemble = d;
    }

    pub fn load_rom(&mut self, file: String) -> Result<(), std::io::Error> {
        let rom = File::open(file)?;
        // TODO: This will enum into RAM to be executed properly
        for (i, b) in rom.bytes().enumerate() {
            println!("Parsing index: {}", i);
            self.parse_opcode(&b.unwrap());
            if i > 10 {
                break;
            }
        }

        Ok(())
    }

    // This will parse the opcode, printing a disassembly if asked
    //
    // TODO: I really think this can be worked into "run_opcode(...)"
    //
    // Parameters:
    //   byte: &u8
    pub fn parse_opcode(&self, byte: &u8) {
        //TODO: This can be a slice of bytes, up to 3 depending on the current need
        let i = match byte {
            0x00 => self.op_00(),  // NOP
            0xC3 => self.op_jmp(), // JMP
            _ => self.op_unk(),    // UNK
        };

        println!("\t{:#04X}\tCode:{}\tOP Size:{}", byte, i.code, i.size);
    }

    pub fn op_00(&self) -> Instr {
        Instr {
            code: "NOP".to_string(),
            size: 2,
        }
    }

    pub fn op_jmp(&self) -> Instr {
        Instr {
            code: "JMP".to_string(),
            size: 3,
        }
    }

    pub fn op_unk(&self) -> Instr {
        Instr {
            code: "!UNK!".to_string(),
            size: 2,
        }
    }
}

pub struct Instr {
    code: String,
    size: usize,
}
