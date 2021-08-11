use std::fs::File;
use std::io::prelude::*;

use super::disassembler;

// Memory related constants
pub const RAM_SIZE: usize = 0xFFFF;
pub const RAM_WORK_START: usize = 0x2000;
pub const RAM_VIDEO_START: usize = 0x2400;
pub const RAM_MIRROR_START: usize = 0x4000;

pub const OPCODE_SIZE: usize = 1;

pub enum ProgramCounter {
    Next,        // The operation does not use any data
    Two,         // The operation uses only 1 byte of data
    Three,       // The operation uses the full 2 bytes of data
    Jump(usize), // The operation jumps to a point in memory
}

pub struct Cpu {
    // Memory
    pub memory: [u8; RAM_SIZE],

    // Registers
    pub pc: usize, // Program Counter
    pub sp: u16,   // Stack Pointer
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

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
            sp: 0x00,
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            disassemble: false,
            nop: false,
        }
    }

    pub fn get_registers(&self) -> (usize, u16, u8, u8) {
        (self.pc, self.sp, self.h, self.l)
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

        // If needed/wanted, call off to the disassembler to print some pretty details
        if self.disassemble {
            disassembler::disassemble(opcode, self.get_registers());
        }

        // D8 = 8 bits (1st byte = y)
        // D16 = 16 bits (1st (y) and 2nd byte (x))
        let i = match opcode.0 {
            0x00 => self.op_00(),       // NOP
            0x06 => self.op_06(x),      // MVI B, D8
            0x11 => self.op_11(x, y),   // LXI D,D16
            0x1A => self.op_1a(),       // LDAX D
            0x21 => self.op_21(x, y),   // LXI D,D16
            0x31 => self.op_31(x, y),   // LXI SP, D16
            0x77 => self.op_77(),       // MOV M,A
            0xC3 => self.op_c3(x, y),   // JMP
            0xC5 => self.op_c5(),       // PUSH B
            0xCD => self.op_cd(x, y),   // CALL Addr
            0xD5 => self.op_d5(),       // PUSH D
            0xE5 => self.op_e5(),       // PUSH H
            0xF5 => self.op_f5(),       // PUSH PSW
            _ => self.op_unk(opcode.0), // UNK
        };

        match i {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Two => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::Three => self.pc += OPCODE_SIZE * 3,
            ProgramCounter::Jump(d) => self.pc = d,
        }
    }

    pub fn op_unk(&self, o: u8) -> ProgramCounter {
        println!("!!OPCODE: {:#04X} is unknown!! BITMASK: {:#010b}", o, o);
        ProgramCounter::Next
    }

    pub fn op_00(&self) -> ProgramCounter {
        ProgramCounter::Next
    }

    // B <- x
    pub fn op_06(&mut self, x: u8) -> ProgramCounter {
        self.b = x;
        ProgramCounter::Two
    }

    // LXI D, D16
    pub fn op_11(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.d = y;
        self.e = x;
        ProgramCounter::Three
    }

    // LDAX DE
    pub fn op_1a(&mut self) -> ProgramCounter {
        let loc: u16 = u16::from(self.d) << 8 | u16::from(self.e);

        self.a = match self.memory.get(loc as usize) {
            Some(&v) => v,
            None => 0,
        };

        ProgramCounter::Next
    }

    //LXI H,D16
    pub fn op_21(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.h = y;
        self.l = x;
        ProgramCounter::Three
    }

    // Load Stack Pointer with the value (y<<8|x)
    // SP.hi <- byte 3, SP.lo <- byte 2
    pub fn op_31(&mut self, x: u8, y: u8) -> ProgramCounter {
        self.sp = u16::from(y) << 8 | u16::from(x);
        ProgramCounter::Three
    }

    // MOV M,A
    // Address specified by H and L registers.
    // Load the value of A into this address in memory.
    fn op_77(&mut self) -> ProgramCounter {
        let loc: u16 = u16::from(self.h) << 8 | u16::from(self.l);
        self.memory[usize::from(loc)] = self.a;
        ProgramCounter::Next
    }

    // Jump to a given location as provided by (y<<8 | x)
    pub fn op_c3(&self, x: u8, y: u8) -> ProgramCounter {
        let ys: u16 = u16::from(y) << 8;
        let dest: u16 = ys | u16::from(x);
        ProgramCounter::Jump(dest.into())
    }

    // (sp-2)<-C; (sp-1)<-B; sp <- sp - 2
    pub fn op_c5(&mut self) -> ProgramCounter {
        self.memory[usize::from(self.sp - 2)] = self.c;
        self.memory[usize::from(self.sp - 1)] = self.b;
        self.sp -= 2;
        ProgramCounter::Next
    }

    // (SP-1)<-PC.hi;(SP-2)<-PC.lo;SP<-SP+2;PC=adr
    pub fn op_cd(&mut self, x: u8, y: u8) -> ProgramCounter {
        // Save away the current PC hi/lo into the stack
        let pc_hi = self.pc >> 4;
        let pc_lo = self.pc & 0x0F;
        self.memory[usize::from(self.sp - 1)] = pc_hi as u8;
        self.memory[usize::from(self.sp - 2)] = pc_lo as u8;
        self.sp += 2;

        // Tell the program counter where we want to go next
        let ys: u16 = u16::from(y) << 8;
        self.pc = usize::from(ys | u16::from(x));

        ProgramCounter::Jump(self.pc)
    }

    pub fn op_d5(&self) -> ProgramCounter {
        ProgramCounter::Next
    }

    pub fn op_e5(&self) -> ProgramCounter {
        ProgramCounter::Next
    }

    pub fn op_f5(&self) -> ProgramCounter {
        ProgramCounter::Next
    }
}
