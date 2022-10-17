#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unused_self, clippy::cast_possible_truncation)]
//pub mod cpu {
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

pub use crate::constants::*;
pub use crate::cpu::common::*;
pub use crate::cpu::opcodes::*;

pub enum ProgramCounter {
    Next,        // The operation does not use any data
    Two,         // The operation uses only 1 byte of data
    Three,       // The operation uses the full 2 bytes of data
    Jump(usize), // The operation jumps to a point in memory
}

#[derive(Clone, Copy)]
pub enum Registers {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC, // A register pair
    DE, // A register pair
    HL, // A register pair, used to reference memory locations
    SP, // Stack pointer
    SW, // Program Status Word
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Registers::A => write!(f, "A"),
            Registers::B => write!(f, "B"),
            Registers::C => write!(f, "C"),
            Registers::D => write!(f, "D"),
            Registers::E => write!(f, "E"),
            Registers::H => write!(f, "H"),
            Registers::L => write!(f, "L"),
            Registers::BC => write!(f, "BC"),
            Registers::DE => write!(f, "DE"),
            Registers::HL => write!(f, "HL"),
            Registers::SP => write!(f, "SP"),
            Registers::SW => write!(f, "SW"),
        }
    }
}

#[derive(Clone)]
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

    // Flags Z,S,P,AC
    pub flags: u8,

    // A flag that indicates we wish to print human readable command references
    pub disassemble: bool,
    // A flag to indicate that we do not wish to execute, probably just printing disassembly
    pub nop: bool,

    pub interrupts: bool, // A flag to indicate we respond to interrupts (see: opcodes EI/DI)

    pub cycle_count: usize,        // Cycle count
    pub last_opcode: (u8, u8, u8), // Just a record of the last opcode.
    pub next_opcode: (u8, u8, u8), // Next opcode we are running.
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CYCLES:{:#08X} PC:{:#06X} SP:{:#06X}\nA:{:#06X}\nB:{:#04X} C:{:#04X}\nD:{:#04X} E:{:#04X}\nH:{:#04X} L:{:#04X}\nsp $[{:#06X}]={:#04X} sp+1 $[{:06X}]={:#04X}",
            self.cycle_count, self.pc, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l,self.sp,self.memory[usize::from(self.sp)],self.sp+1,self.memory[usize::from(self.sp+1)]
        )
    }
}

impl Cpu {
    #[must_use]
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
            flags: 0x02, // 00000010 is the default starting point
            disassemble: false,
            nop: false,
            interrupts: false,
            cycle_count: 0x00,
            last_opcode: (0, 0, 0),
            next_opcode: (0, 0, 0),
        }
    }

    // Returns a usize location in memory designed by the H and L registers
    pub fn get_addr_pointer(&mut self) -> usize {
        usize::from(u16::from(self.h) << 8 | u16::from(self.l))
    }

    #[must_use]
    pub fn get_registers(&self) -> (&usize, &u16, &u8, &u8, &u8) {
        (&self.pc, &self.sp, &self.h, &self.l, &self.b)
    }

    // Returns a paired register such as HL or BC.
    // Pass to the function the beginning register for the pair
    // Returned value will be a u16 value
    #[must_use]
    pub fn get_register_pair(&self, register: Registers) -> u16 {
        match register {
            Registers::BC => u16::from(self.b) << 8 | u16::from(self.c),
            Registers::DE => u16::from(self.d) << 8 | u16::from(self.e),
            Registers::HL => u16::from(self.h) << 8 | u16::from(self.l),
            Registers::SP => self.sp,
            _ => 0_u16,
        }
    }

    // Sets a register pair if appropriate
    pub fn set_register_pair(&mut self, register: Registers, val: u16) {
        let h: u8 = (val >> 8) as u8;
        let l: u8 = (val & 0xff) as u8;
        match register {
            Registers::BC => {
                self.b = h;
                self.c = l;
            }
            Registers::DE => {
                self.d = h;
                self.e = l;
            }
            Registers::HL => {
                self.h = h;
                self.l = l;
            }
            Registers::SP => {
                self.sp = val;
            }
            _ => (),
        };
    }

    // Returns the current flag values
    #[must_use]
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    // Returns true if a flag is set
    pub fn test_flag(&mut self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    // Returns the binary value of a flag, as a u8 for various ops.
    pub fn get_flag(&mut self, mask: u8) -> u8 {
        u8::from(self.test_flag(mask))
    }

    // Sets a flag using a bitwise OR operation
    // Mask of 2 (00100) with a value of 1 = 00100
    // if flags = 10010 new value will be 10110
    pub fn set_flag(&mut self, mask: u8) {
        self.flags |= mask;
    }

    // Resets a flag using bitwise AND operation
    // Mask of 2 (00100) with a value of 0 = 11011
    // if flags = 11111 new value will be 11011
    pub fn reset_flag(&mut self, mask: u8) {
        self.flags &= !mask;
    }

    // Computes and sets the mask of flags for a supplied value
    // sets flags: Zero, Sign, Parity, Carry, and Auxiliary Carry
    // If provided, it will also set Overflow and Aux_Carry, resetting them otherwise
    pub fn update_flags(&mut self, val: u8, overflow: Option<bool>, aux_carry: Option<bool>) {
        if val == 0 {
            self.set_flag(FLAG_ZERO);
        } else {
            self.reset_flag(FLAG_ZERO);
        }

        if get_sign(val) {
            self.set_flag(FLAG_SIGN); // A negative number
        } else {
            self.reset_flag(FLAG_SIGN); // A positive number
        }

        if get_parity(val.into()) {
            self.set_flag(FLAG_PARITY);
        } else {
            self.reset_flag(FLAG_PARITY);
        }

        if let Some(of) = overflow {
            if of {
                self.set_flag(FLAG_CARRY);
            } else {
                self.reset_flag(FLAG_CARRY);
            }
        };

        if let Some(ac) = aux_carry {
            if ac {
                self.set_flag(FLAG_AUXCARRY);
            } else {
                self.reset_flag(FLAG_AUXCARRY);
            }
        };
    }

    pub fn set_disassemble(&mut self, d: bool) {
        self.disassemble = d;
    }

    pub fn set_nop(&mut self, n: bool) {
        self.nop = n;
    }

    /// Load the ROM file into memory, starting at ``start_index``
    /// Returns a tuple containing the index we started at and where we
    /// actually finished at.
    ///
    /// # Errors
    /// Will return a standard io Error if necessary
    /// # Panics
    /// If the error happens, this will cause the function to panic
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

    /// Gathers a word from memory based on program counter location,
    /// then passes it along to the ``run_opcode()`` function
    /// On successful tick, returns the program counter value that was run
    /// On unsuccessful tick, returns an error
    ///
    /// # Errors
    /// Will return an error if necessary
    /// # Panics
    /// Will panic if an error happens
    pub fn tick(&mut self) -> Result<usize, String> {
        let opcode = self.read_opcode();    // Gather the current opcode to run, based on PC's location
        self.last_opcode = opcode;
        let this_pc = self.pc;

        // If we are in a STOPPED state, no action is necessary
        // This will be "unstopped" when an interrupt occurs
        if self.nop {
            return Ok(this_pc);
        }

        self.cycle_count += 1;

        match self.run_opcode(opcode) {
            Ok(_) => {
                self.next_opcode = self.read_opcode();
                Ok(this_pc)
            }
            Err(e) => Err(e),
        }
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

    /// This processes the opcodes beginning with the pattern "0X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_0x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0x00 => ProgramCounter::Next,               // NOP
            0x01 => self.op_lxi(Registers::BC, dl, dh), // LXI B,D16
            0x02 => self.op_stax(Registers::BC),        // STAX (BC)
            0x03 => self.op_inx(Registers::BC),         // INX B
            0x04 => self.op_inr(Registers::B),          // INR B
            0x05 => self.op_dcr(Registers::B),          // DCR B
            0x06 => self.op_mvi(Registers::B, dl),      // MVI B, D8
            0x07 => self.op_rlc_ral(false),             // RLC (Rotate left)
            //0x08
            0x09 => self.op_dad(Registers::B),     // DAD BC
            0x0A => self.op_ldax(Registers::BC),   // LDAX BC
            0x0B => self.op_dcx(Registers::BC),    // DCX BC
            0x0C => self.op_inr(Registers::C),     // INR C
            0x0D => self.op_dcr(Registers::C),     // DCR D
            0x0E => self.op_mvi(Registers::C, dl), // MVI C, D8
            0x0F => self.op_rrc_rar(false),        // RRC
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "1X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_1x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            //0x10
            0x11 => self.op_lxi(Registers::DE, dl, dh), // LXI D,D16
            0x12 => self.op_stax(Registers::DE),        // STAX (DE)
            0x13 => self.op_inx(Registers::DE),         // INX DE
            0x14 => self.op_inr(Registers::D),          // INR D
            0x15 => self.op_dcr(Registers::D),          // DCR D
            0x16 => self.op_mvi(Registers::D, dl),      // MVI D
            0x17 => self.op_rlc_ral(true),              // RAL (Rotate left through carry)
            0x19 => self.op_dad(Registers::D),          // DAD D
            0x1A => self.op_ldax(Registers::DE),        // LDAX DE
            0x1B => self.op_dcx(Registers::DE),         // DCX DE
            0x1C => self.op_inr(Registers::E),          // INR E
            0x1D => self.op_dcr(Registers::E),          // DCR E
            0x1E => self.op_mvi(Registers::E, dl),      // MVI E
            0x1F => self.op_rrc_rar(true),              // RAR
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "2X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_2x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0x21 => self.op_lxi(Registers::HL, dl, dh), // LXI X,D16
            0x23 => self.op_inx(Registers::HL),         // INX HL
            0x24 => self.op_inr(Registers::H),          // INR H
            0x25 => self.op_dcr(Registers::H),          // DCR H
            0x26 => self.op_mvi(Registers::H, dl),      // MVI H, D8
            0x27 => self.op_daa(),                      // DAA
            0x29 => self.op_dad(Registers::H),          // DAD HL
            0x2A => self.lhld(dl, dh),                  // LDA DL DH
            0x2B => self.op_dcx(Registers::HL),         // DCX HL
            0x2C => self.op_inr(Registers::L),          // INR L
            0x2D => self.op_dcr(Registers::L),          // DCR L
            0x2E => self.op_mvi(Registers::L, dl),      // MVI L
            0x2F => self.op_comp(Registers::A),         // CMA
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "3X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_3x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0x31 => self.op_lxi(Registers::SP, dl, dh), // LXI SP, D16
            0x32 => self.op_sta(dl, dh),                // STA (adr)<-A
            0x33 => self.op_inx(Registers::SP),         // INX SP
            0x34 => self.op_inr(Registers::HL),         // INR (HL)
            0x35 => self.op_dcr(Registers::HL),         // DCR (HL)
            0x36 => self.op_mvi(Registers::HL, dl),     // MVI (HL)<-D8
            0x37 => self.op_stc(),                      // STC
            0x39 => self.op_dad(Registers::SP),         // DAD SP
            0x3A => self.op_lda(dl, dh),                // LDA adr
            0x3B => self.op_dcx(Registers::SP),         // DCX SP
            0x3C => self.op_inr(Registers::A),          // INR A
            0x3D => self.op_dcr(Registers::A),          // DCR A
            0x3E => self.op_mvi(Registers::A, dl),      // MVI A
            0x3F => self.op_cmc(),                      // CMC
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "4X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_4x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0x40 => self.op_mov(Registers::B, Registers::B), // MOV B <- B
            0x41 => self.op_mov(Registers::B, Registers::C), // MOV B <- C
            0x42 => self.op_mov(Registers::B, Registers::D), // MOV B <- D
            0x43 => self.op_mov(Registers::B, Registers::E), // MOV B <- E
            0x44 => self.op_mov(Registers::B, Registers::H), // MOV B <- H
            0x45 => self.op_mov(Registers::B, Registers::L), // MOV B <- L
            0x46 => self.op_mov(Registers::B, Registers::HL), // MOV B <- (HL)
            0x47 => self.op_mov(Registers::B, Registers::A), // MOV B <- A
            0x48 => self.op_mov(Registers::C, Registers::B), // MOV C <- B
            0x49 => self.op_mov(Registers::C, Registers::C), // MOV C <- C
            0x4A => self.op_mov(Registers::C, Registers::D), // MOV C <- D
            0x4B => self.op_mov(Registers::C, Registers::E), // MOV C <- E
            0x4C => self.op_mov(Registers::C, Registers::H), // MOV C <- H
            0x4D => self.op_mov(Registers::C, Registers::L), // MOV C <- L
            0x4E => self.op_mov(Registers::C, Registers::HL), // MOV C <- HL
            0x4F => self.op_mov(Registers::C, Registers::A), // MOV C <- A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "5X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_5x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0x50 => self.op_mov(Registers::D, Registers::B), // MOV D <- B
            0x51 => self.op_mov(Registers::D, Registers::C), // MOV D <- C
            0x52 => self.op_mov(Registers::D, Registers::D), // MOV D <- D
            0x53 => self.op_mov(Registers::D, Registers::E), // MOV D <- E
            0x54 => self.op_mov(Registers::D, Registers::H), // MOV D <- H
            0x55 => self.op_mov(Registers::D, Registers::L), // MOV D <- L
            0x56 => self.op_mov(Registers::D, Registers::HL), // MOV D <- (HL)
            0x57 => self.op_mov(Registers::D, Registers::A), // MOV D <- A
            0x58 => self.op_mov(Registers::E, Registers::B), // MOV E <- B
            0x59 => self.op_mov(Registers::E, Registers::C), // MOV E <- C
            0x5A => self.op_mov(Registers::E, Registers::D), // MOV E <- D
            0x5B => self.op_mov(Registers::E, Registers::E), // MOV E <- E
            0x5C => self.op_mov(Registers::E, Registers::H), // MOV E <- H
            0x5D => self.op_mov(Registers::E, Registers::L), // MOV E <- L
            0x5E => self.op_mov(Registers::E, Registers::HL), // MOV E <- HL
            0x5F => self.op_mov(Registers::E, Registers::A), // MOV E <- A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "6X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_6x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0x60 => self.op_mov(Registers::H, Registers::B), // MOV H <- B
            0x61 => self.op_mov(Registers::H, Registers::C), // MOV H <- C
            0x62 => self.op_mov(Registers::H, Registers::D), // MOV H <- D
            0x63 => self.op_mov(Registers::H, Registers::E), // MOV H <- E
            0x64 => self.op_mov(Registers::H, Registers::H), // MOV H <- H
            0x65 => self.op_mov(Registers::H, Registers::L), // MOV H <- L
            0x66 => self.op_mov(Registers::H, Registers::HL), // MOV H <- (HL)
            0x67 => self.op_mov(Registers::H, Registers::A), // MOV H <- A
            0x68 => self.op_mov(Registers::L, Registers::B), // MOV L <- B
            0x69 => self.op_mov(Registers::L, Registers::C), // MOV L <- C
            0x6A => self.op_mov(Registers::L, Registers::D), // MOV L <- D
            0x6B => self.op_mov(Registers::L, Registers::E), // MOV L <- E
            0x6C => self.op_mov(Registers::L, Registers::H), // MOV L <- H
            0x6D => self.op_mov(Registers::L, Registers::L), // MOV L <- L
            0x6E => self.op_mov(Registers::L, Registers::HL), // MOV L <- HL
            0x6F => self.op_mov(Registers::L, Registers::A), // MOV L <- A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "7X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_7x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0x70 => self.op_mov(Registers::HL, Registers::B), // MOV M,B	1		(HL) <- B
            0x71 => self.op_mov(Registers::HL, Registers::C), // MOV M,C	1		(HL) <- C
            0x72 => self.op_mov(Registers::HL, Registers::D), // MOV M,D	1		(HL) <- D
            0x73 => self.op_mov(Registers::HL, Registers::E), // MOV M,E	1		(HL) <- E
            0x74 => self.op_mov(Registers::HL, Registers::H), // MOV M,H	1		(HL) <- H
            0x75 => self.op_mov(Registers::HL, Registers::L), // MOV M,L	1		(HL) <- L
            0x76 => self.op_hlt(),                            // HLT 1 (special)
            0x77 => self.op_mov(Registers::HL, Registers::A), // MOV M,A
            0x78 => self.op_mov(Registers::A, Registers::B),  // MOV A,B
            0x79 => self.op_mov(Registers::A, Registers::C),  // MOV A,C
            0x7A => self.op_mov(Registers::A, Registers::D),  // MOV A,D
            0x7B => self.op_mov(Registers::A, Registers::E),  // MOV A,E
            0x7C => self.op_mov(Registers::A, Registers::H),  // MOV A,H
            0x7D => self.op_mov(Registers::A, Registers::L),  // MOV A,L
            0x7E => self.op_mov(Registers::A, Registers::HL), // MOV A,(HL)
            0x7F => self.op_mov(Registers::A, Registers::A),  // MOV A,A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "8X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_8x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0x80 => self.op_add(Registers::B),  // ADD B
            0x81 => self.op_add(Registers::C),  // ADD C
            0x82 => self.op_add(Registers::D),  // ADD D
            0x83 => self.op_add(Registers::E),  // ADD E
            0x84 => self.op_add(Registers::H),  // ADD H
            0x85 => self.op_add(Registers::L),  // ADD L
            0x86 => self.op_add(Registers::HL), // ADD M
            0x87 => self.op_add(Registers::A),  // ADD A
            0x88 => self.op_adc(Registers::B),  // ADC B
            0x89 => self.op_adc(Registers::C),  // ADC C
            0x8A => self.op_adc(Registers::D),  // ADC D
            0x8B => self.op_adc(Registers::E),  // ADC E
            0x8C => self.op_adc(Registers::H),  // ADC H
            0x8D => self.op_adc(Registers::L),  // ADC L
            0x8E => self.op_adc(Registers::HL), // ADC M
            0x8F => self.op_adc(Registers::A),  // ADC A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "9X"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_9x(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let sub = self.get_flag(FLAG_CARRY);
        let i = match opcode.0 {
            0x90 => self.op_sub(Registers::B, 0),    // SUB B
            0x91 => self.op_sub(Registers::C, 0),    // SUB C
            0x92 => self.op_sub(Registers::D, 0),    // SUB D
            0x93 => self.op_sub(Registers::E, 0),    // SUB E
            0x94 => self.op_sub(Registers::H, 0),    // SUB H
            0x95 => self.op_sub(Registers::L, 0),    // SUB L
            0x96 => self.op_sub(Registers::HL, 0),   // SUB M
            0x97 => self.op_sub(Registers::A, 0),    // SUB A
            0x98 => self.op_sub(Registers::B, sub),  // SBB B
            0x99 => self.op_sub(Registers::C, sub),  // SBB B
            0x9A => self.op_sub(Registers::D, sub),  // SBB B
            0x9B => self.op_sub(Registers::E, sub),  // SBB B
            0x9C => self.op_sub(Registers::H, sub),  // SBB B
            0x9D => self.op_sub(Registers::L, sub),  // SBB B
            0x9E => self.op_sub(Registers::HL, sub), // SBB B
            0x9F => self.op_sub(Registers::A, sub),  // SBB B
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "AX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_ax(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0xA0 => self.op_ana(Registers::B),  // ANA B
            0xA1 => self.op_ana(Registers::C),  // ANA C
            0xA2 => self.op_ana(Registers::D),  // ANA D
            0xA3 => self.op_ana(Registers::E),  // ANA E
            0xA4 => self.op_ana(Registers::H),  // ANA H
            0xA5 => self.op_ana(Registers::L),  // ANA L
            0xA6 => self.op_ana(Registers::HL), // ANA HL
            0xA7 => self.op_ana(Registers::A),  // ANA A
            0xA8 => self.op_xra(Registers::B),  // XRA B
            0xA9 => self.op_xra(Registers::C),  // XRA C
            0xAA => self.op_xra(Registers::D),  // XRA D
            0xAB => self.op_xra(Registers::E),  // XRA E
            0xAC => self.op_xra(Registers::H),  // XRA H
            0xAD => self.op_xra(Registers::L),  // XRA L
            0xAE => self.op_xra(Registers::HL), // XRA HL
            0xAF => self.op_xra(Registers::A),  // XRA A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "BX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_bx(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let i = match opcode.0 {
            0xB0 => self.op_ora(Registers::B),  // ORA B
            0xB1 => self.op_ora(Registers::C),  // ORA C
            0xB2 => self.op_ora(Registers::D),  // ORA D
            0xB3 => self.op_ora(Registers::E),  // ORA E
            0xB4 => self.op_ora(Registers::H),  // ORA H
            0xB5 => self.op_ora(Registers::L),  // ORA L
            0xB6 => self.op_ora(Registers::HL), // ORA (HL)
            0xB7 => self.op_ora(Registers::A),  // ORA A
            0xB8 => self.op_cmp(Registers::B),  // CMP B
            0xB9 => self.op_cmp(Registers::C),  // CMP C
            0xBA => self.op_cmp(Registers::D),  // CMP D
            0xBB => self.op_cmp(Registers::E),  // CMP E
            0xBC => self.op_cmp(Registers::H),  // CMP H
            0xBD => self.op_cmp(Registers::L),  // CMP L
            0xBE => self.op_cmp(Registers::HL), // CMP (HL)
            0xBF => self.op_cmp(Registers::A),  // CMP A
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "CX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_cx(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction
        let flag_carry = self.test_flag(FLAG_CARRY);

        let i = match opcode.0 {
            0xC0 => self.op_rets(FLAG_ZERO, false),            // RNZ
            0xC1 => self.op_pop(Registers::B),                 // POP B
            0xC2 => self.op_jnz(dl, dh),                       // JNZ
            0xC3 => Cpu::op_jmp(dl, dh),                       // JMP
            0xC4 => self.op_call_if(FLAG_ZERO, false, dl, dh), // CNZ
            0xC5 => self.op_push(Registers::B),                // PUSH B
            0xC6 => self.op_adi_aci(dl, false),                // ADI
            0xC7 => self.op_rst(0b000),                        // RST 0
            0xC8 => self.op_rets(FLAG_CARRY, true),            // RC
            0xC9 => self.op_ret(),                             // RET
            0xCA => self.op_jz(dl, dh),                        // JZ
            // 0xCB
            0xCC => self.op_call_if(FLAG_ZERO, true, dl, dh), // CZ
            0xCD => self.op_call(dl, dh),                     // CALL Addr
            0xCE => self.op_adi_aci(dl, flag_carry),          // ACI
            0xCF => self.op_rst(0b001),                       // RST 1
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "DX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_dx(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0xD0 => self.op_rets(FLAG_CARRY, false),            // RNC
            0xD1 => self.op_pop(Registers::D),                  // POP D
            0xD2 => self.op_jnc(dl, dh),                        // JNC
            0xD3 => self.op_out(dl),                            // OUT
            0xD4 => self.op_call_if(FLAG_CARRY, false, dl, dh), // CNC
            0xD5 => self.op_push(Registers::D),                 // PUSH D
            0xD7 => self.op_rst(0b010),                         // RST 2
            0xDC => self.op_call_if(FLAG_CARRY, true, dl, dh),  // CC
            0xDF => self.op_rst(0b011),                         // RST 3
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "EX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_ex(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0xE0 => self.op_rets(FLAG_PARITY, false),            // RPO
            0xE1 => self.op_pop(Registers::H),                   // POP H
            0xE4 => self.op_call_if(FLAG_PARITY, false, dl, dh), // CPO
            0xE5 => self.op_push(Registers::H),                  // PUSH H
            0xE6 => self.op_ani(dl),                             // ANI (And with Accum, Immediate)
            0xE7 => self.op_rst(0b100),                          // RST 4
            0xE8 => self.op_rets(FLAG_PARITY, true),             // RPE
            0xEB => self.op_xchg(),                              // XCHG
            0xEC => self.op_call_if(FLAG_PARITY, true, dl, dh),  // CPE
            0xEF => self.op_rst(0b101),                          // RST 5
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This processes the opcodes beginning with the pattern "FX"
    ///
    /// # Errors
    /// Will return ERROR if opcode was not recognized
    pub fn opcodes_fx(&mut self, opcode: (u8, u8, u8)) -> Result<ProgramCounter, String> {
        let dl = opcode.1; // Potential data points for usage by an instruction
        let dh = opcode.2; // Potential data points for usage by an instruction

        let i = match opcode.0 {
            0xF0 => self.op_rets(FLAG_SIGN, false), // RP
            0xF1 => self.op_pop(Registers::SW),     // POP SW
            0xF3 => {
                self.interrupts = false;
                ProgramCounter::Next
            } // Disable interrupts
            0xF4 => self.op_call_if(FLAG_SIGN, false, dl, dh), // CP
            0xF5 => self.op_push(Registers::SW),    // Push SW
            0xFE => self.op_cpi(dl),                // CPI
            0xF7 => self.op_rst(0b110),             // RST 6
            0xF8 => self.op_rets(FLAG_SIGN, true),  // RM
            0xFA => self.op_jump_if(FLAG_SIGN, true, dl, dh), // JM
            0xFB => {
                self.interrupts = true;
                ProgramCounter::Next
            } // Enable interrupts
            0xFC => self.op_call_if(FLAG_SIGN, true, dl, dh), // CM
            0xFF => self.op_rst(0b111),             // RST 7
            _ => {
                return Err(format!(
                    "!! OPCODE: {:#04X} {:#010b} is unknown !!",
                    opcode.0, opcode.0
                ))
            }
        };

        Ok(i)
    }

    /// This will parse the opcode, printing a disassembly if asked
    ///
    /// An opcode consists of:
    ///  Instruction (1 byte)
    ///  Data (1 or 2 bytes) depending on opcode.  Little endian.
    ///
    /// # Errors
    /// It will return ERROR if the opcode was not recognized
    pub fn run_opcode(&mut self, opcode: (u8, u8, u8)) -> Result<(), String> {
        // D8 = 8 bits (1st byte = y)
        // D16 = 16 bits (1st (y) and 2nd byte (x))
        let i = match opcode.0 {
            0x00..=0x0F => self.opcodes_0x(opcode),
            0x10..=0x1F => self.opcodes_1x(opcode),
            0x20..=0x2F => self.opcodes_2x(opcode),
            0x30..=0x3F => self.opcodes_3x(opcode),
            0x40..=0x4F => self.opcodes_4x(opcode),
            0x50..=0x5F => self.opcodes_5x(opcode),
            0x60..=0x6F => self.opcodes_6x(opcode),
            0x70..=0x7F => self.opcodes_7x(opcode),
            0x80..=0x8F => self.opcodes_8x(opcode),
            0x90..=0x9F => self.opcodes_9x(opcode),
            0xA0..=0xAF => self.opcodes_ax(opcode),
            0xB0..=0xBF => self.opcodes_bx(opcode),
            0xC0..=0xCF => self.opcodes_cx(opcode),
            0xD0..=0xDF => self.opcodes_dx(opcode),
            0xE0..=0xEF => self.opcodes_ex(opcode),
            0xF0..=0xFF => self.opcodes_fx(opcode),
        };

        match i {
            Ok(ProgramCounter::Next) => self.pc += OPCODE_SIZE,
            Ok(ProgramCounter::Two) => self.pc += OPCODE_SIZE * 2,
            Ok(ProgramCounter::Three) => self.pc += OPCODE_SIZE * 3,
            Ok(ProgramCounter::Jump(d)) => self.pc = d,
            Err(e) => return Err(e),
        }

        Ok(())
    }
}
